use crate::{anyhow, Result, Command, KvsEngine};
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write},
    path::PathBuf,
};
use walkdir::WalkDir;

const SINGLE_FILE_SIZE: u64 = 1024 * 1024;

pub struct KvStore {
    index: HashMap<String, (u64, u64)>,
    path: PathBuf,
    active_nth_file: u64,
}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path: PathBuf = path.into();
        let path_at = |n: u64| path.join("kvs.data.".to_owned() + &n.to_string());

        // rebuild the in-memory index
        let mut index = HashMap::new();

        // if no file exists, set active_nth_file 0
        if !path_at(0).exists() {
            return Ok(KvStore {
                index,
                path,
                active_nth_file: 0,
            });
        }

        // scan how many kvs.data.* files in the given dir
        let mut nfile: u64 = 0;
        for entry in WalkDir::new(&path).min_depth(1).max_depth(1) {
            if entry?
                .file_name()
                .to_string_lossy()
                .starts_with("kvs.data.")
            {
                nfile += 1;
            }
        }

        // read each kvs.data.* file
        for i in 0..nfile {
            let file = File::open(path_at(i))?;
            let reader = BufReader::new(&file);

            // replay each command
            let mut pos: u64 = 0;
            for command in reader.split(b'#') {
                let command = command?;
                let next_pos = pos + command.len() as u64 + 1;

                let command = serde_json::from_slice(&command)?;
                match command {
                    Command::Set { key, .. } => {
                        index.insert(key.clone(), (i, pos));
                    }
                    Command::Rm { key } => {
                        index.remove(&key);
                    }
                    _ => (),
                }
                pos = next_pos;
            }
        }

        Ok(KvStore {
            index,
            path,
            active_nth_file: nfile - 1,
        })
    }

    fn path_at(&self, n: u64) -> PathBuf {
        self.path.join("kvs.data.".to_owned() + &n.to_string())
    }

    fn active_path(&self) -> PathBuf {
        self.path_at(self.active_nth_file)
    }

    // rewrite records to the active file
    fn compact(&mut self) -> Result<()> {
        let active_path = self.active_path();

        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&active_path)?;
        let mut writer = BufWriter::new(file);

        let mut new_index = HashMap::new();
        for (key, (n, mut pos)) in &self.index {
            if *n < self.active_nth_file {
                let command = KvStore::read_command_at(self.path_at(*n), pos)?;
                pos = KvStore::write_command_to_writer(&mut writer, &command)?;
            }

            new_index.insert(key.clone(), (self.active_nth_file, pos));
        }

        self.index = new_index;

        for i in 0..self.active_nth_file {
            fs::remove_file(self.path_at(i))?;
        }
        fs::rename(active_path, self.path_at(0))?;
        self.active_nth_file = 0;

        Ok(())
    }

    fn try_compact(&mut self, last_pos: u64) -> Result<()> {
        // TODO: control the crash
        if last_pos > SINGLE_FILE_SIZE {
            // create new file if the active file is large
            self.active_nth_file += 1;

            if (self.index.len() as u64) < self.active_nth_file * 1024 {
                // compact logs if active records are much less than old records
                self.compact()?;
            }
        }
        Ok(())
    }

    fn read_command_at(path: PathBuf, pos: u64) -> Result<Command> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(pos))?;

        let mut command = Vec::new();
        reader.read_until(b'#', &mut command)?;
        command.pop();

        Ok(serde_json::from_slice(&command)?)
    }

    fn write_command_to(path: PathBuf, command: &Command) -> Result<u64> {
        let file = OpenOptions::new().append(true).create(true).open(path)?;
        KvStore::write_command_to_writer(&mut BufWriter::new(file), command)
    }

    fn write_command_to_writer(writer: &mut BufWriter<File>, command: &Command) -> Result<u64> {
        writer.seek(SeekFrom::End(0))?;
        let pos = writer.stream_position()?;

        let mut buffer = serde_json::to_vec(command)?;
        buffer.push(b'#');

        writer.write_all(&buffer)?;

        Ok(pos)
    }
}

impl KvsEngine for KvStore {
    /// Set the given value with the given key.
    ///
    /// # Examples
    ///
    /// ```
    /// use tempfile::TempDir;
    ///
    /// # fn main() -> kvs::Result<()> {
    /// let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    /// let mut store = kvs::KvStore::open(temp_dir.path())?;
    ///
    /// store.set("k".to_owned(), "v".to_owned())?;
    /// assert_eq!(store.get("k".to_owned())?, Some("v".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let command = Command::Set {
            key: key.clone(),
            value,
        };
        let pos = KvStore::write_command_to(self.active_path(), &command)?;

        self.index.insert(key, (self.active_nth_file, pos));
        self.try_compact(pos)?;

        Ok(())
    }

    /// Get the corresponding value of the given key,
    /// return None if the key not exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use tempfile::TempDir;
    ///
    /// # fn main() -> kvs::Result<()> {
    /// let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    /// let mut store = kvs::KvStore::open(temp_dir.path())?;
    ///
    /// assert_eq!(store.get("k".to_owned())?, None);
    /// store.set("k".to_owned(), "v".to_owned());
    /// assert_eq!(store.get("k".to_owned())?, Some("v".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(&(n, pos)) = self.index.get(&key) {
            match KvStore::read_command_at(self.path_at(n), pos)? {
                Command::Set { key: _, value } => Ok(Some(value)),
                _ => Err(anyhow!("Err Log!!!")),
            }
        } else {
            Ok(None)
        }
    }

    /// Remove the given key and the corresponding value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tempfile::TempDir;
    ///
    /// # fn main() -> kvs::Result<()> {
    /// let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    /// let mut store = kvs::KvStore::open(temp_dir.path())?;
    ///
    /// store.set("k".to_owned(), "v".to_owned());
    /// assert_eq!(store.get("k".to_owned())?, Some("v".to_owned()));
    /// store.remove("k".to_owned());
    /// assert_eq!(store.get("k".to_owned())?, None);
    /// # Ok(())
    /// # }
    /// ```
    fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let command = Command::Rm { key: key.clone() };
            let pos = KvStore::write_command_to(self.active_path(), &command)?;

            self.index.remove(&key);
            self.try_compact(pos)?;

            Ok(())
        } else {
            Err(anyhow!("Key not found"))
        }
    }
}
