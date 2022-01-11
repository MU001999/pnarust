use super::KvsEngine;
use crate::{Command, Error, Result};
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
    log_readers: Vec<BufReader<File>>,
    active_nth_file: u64,
    active_writer: BufWriter<File>,
    unused: usize,
}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path: PathBuf = path.into();
        let path_at = |n: u64| path.join("kvs.data.".to_owned() + &n.to_string());

        if !path.exists() {
            fs::create_dir(&path)?;
        }

        // rebuild the in-memory index
        let mut index = HashMap::new();
        let mut log_readers = Vec::new();
        let mut unused = 0;

        // if no file exists, set active_nth_file 0
        let active_nth_file = if !path_at(0).exists() {
            File::create(path_at(0))?;
            log_readers.push(BufReader::new(File::open(path_at(0))?));
            0
        } else {
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
                            if index.insert(key.clone(), (i, pos)).is_some() {
                                unused += 1;
                            };
                        }
                        Command::Rm { key } => {
                            index.remove(&key);
                            unused += 1;
                        }
                        _ => (),
                    }
                    pos = next_pos;
                }

                log_readers.push(BufReader::new(file));
            }

            nfile - 1
        };

        let active_writer = BufWriter::new(
            OpenOptions::new()
                .append(true)
                .open(path_at(active_nth_file))?,
        );

        Ok(KvStore {
            index,
            path,
            log_readers,
            active_nth_file,
            active_writer,
            unused,
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
        let mut new_index = HashMap::new();
        for (key, (n, mut pos)) in &self.index {
            if *n < self.active_nth_file {
                let reader = &mut self.log_readers[*n as usize];
                let command = KvStore::read_command_from(reader, pos)?;
                pos = KvStore::write_command_to(&mut self.active_writer, &command)?;
            }

            new_index.insert(key.clone(), (0, pos));
        }
        self.active_writer.flush()?;

        self.log_readers.clear();
        for i in 0..self.active_nth_file {
            fs::remove_file(self.path_at(i))?;
        }
        fs::rename(self.active_path(), self.path_at(0))?;

        self.active_nth_file = 0;
        self.index = new_index;
        self.log_readers
            .push(BufReader::new(File::open(self.active_path())?));
        self.active_writer =
            BufWriter::new(OpenOptions::new().append(true).open(self.active_path())?);
        self.unused = 0;

        Ok(())
    }

    fn try_compact(&mut self, last_pos: u64) -> Result<()> {
        // TODO: control the crash
        if last_pos > SINGLE_FILE_SIZE {
            // create new file if the active file is large
            self.active_nth_file += 1;
            self.active_writer = BufWriter::new(
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(self.active_path())?,
            );
            self.log_readers
                .push(BufReader::new(File::open(self.active_path())?));

            if self.unused > 1024 {
                // compact logs if active records are much less than old records
                self.compact()?;
            }
        }
        Ok(())
    }

    fn read_command(&mut self, nfile: u64, pos: u64) -> Result<Command> {
        let reader = &mut self.log_readers[nfile as usize];
        KvStore::read_command_from(reader, pos)
    }

    fn read_command_from(reader: &mut BufReader<File>, pos: u64) -> Result<Command> {
        reader.seek(SeekFrom::Start(pos))?;

        let mut command = Vec::new();
        reader.read_until(b'#', &mut command)?;
        command.pop();

        Ok(serde_json::from_slice(&command)?)
    }

    fn write_command(&mut self, command: &Command) -> Result<u64> {
        KvStore::write_command_to(&mut self.active_writer, command)
    }

    fn write_command_to(writer: &mut BufWriter<File>, command: &Command) -> Result<u64> {
        writer.seek(SeekFrom::End(0))?;
        let pos = writer.stream_position()?;

        serde_json::to_writer(&mut *writer, command)?;
        writer.write_all(b"#")?;

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
    /// use kvs::KvsEngine;
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
        let pos = self.write_command(&command)?;
        self.active_writer.flush()?;

        if self
            .index
            .insert(key, (self.active_nth_file, pos))
            .is_some()
        {
            self.unused += 1;
        };
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
    /// use kvs::KvsEngine;
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
            match self.read_command(n, pos)? {
                Command::Set { key: _, value } => Ok(Some(value)),
                _ => Err(Error::ErrorLogMeet),
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
    /// use kvs::KvsEngine;
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
        match self.index.remove(&key) {
            Some(_) => {
                let pos = self.write_command(&Command::Rm { key: key.clone() })?;
                self.active_writer.flush()?;

                self.unused += 1;
                self.try_compact(pos)?;

                Ok(())
            }
            None => Err(Error::KeyNotFound),
        }
    }
}
