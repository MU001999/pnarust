use super::KvsEngine;
use crate::{Command, Error, Result};

use std::{
    collections::HashMap,
    fs::{self, copy, File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicPtr, Ordering},
        Arc, Mutex,
    },
};
use walkdir::WalkDir;

const SINGLE_FILE_SIZE: u64 = 1024 * 1024;
const UNUSED_LIMIT: usize = 1024;

struct KvStoreReader(HashMap<String, (u64, u64)>);

impl KvStoreReader {
    fn raw_arc(index: HashMap<String, (u64, u64)>) -> *mut Arc<KvStoreReader> {
        Box::into_raw(Box::new(Arc::new(KvStoreReader(index))))
    }
}

struct KvStoreWriter {
    base_file: u64,
    active_file: u64,
    active_writer: BufWriter<File>,
    unused: usize,
}

/// A store engine that allows lock-free readers to read.
pub struct KvStore {
    reader: Arc<AtomicPtr<Arc<KvStoreReader>>>,
    writer: Arc<Mutex<KvStoreWriter>>,
    path: PathBuf,
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
        let mut unused = 0;

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

        // if no file exists, set active_file 0
        let (active_file, base_file) = if nfile == 0 {
            (0, 0)
        } else {
            let mut base = 0;
            base = loop {
                if path_at(base).is_file() {
                    break base;
                } else {
                    base += 1;
                }
            };

            // read each kvs.data.* file
            for i in base..base + nfile {
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
                            }
                        }
                        Command::Rm { key } => {
                            index.remove(&key);
                            unused += 1;
                        }
                        _ => (),
                    }
                    pos = next_pos;
                }
            }

            (base + nfile - 1, base)
        };

        let active_writer = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path_at(active_file))?,
        );

        let reader = Arc::new(AtomicPtr::new(KvStoreReader::raw_arc(index)));

        let writer = KvStoreWriter {
            base_file,
            active_file,
            active_writer,
            unused,
        };
        let writer = Arc::new(Mutex::new(writer));

        Ok(KvStore {
            reader,
            writer,
            path,
        })
    }

    fn get_reader(&self) -> Arc<KvStoreReader> {
        unsafe { Arc::clone(&(*self.reader.load(Ordering::Relaxed))) }
    }

    fn swap_index(&self, index: HashMap<String, (u64, u64)>) {
        let old = self
            .reader
            .swap(KvStoreReader::raw_arc(index), Ordering::Relaxed);
        unsafe {
            drop(Box::from_raw(old));
        }
    }

    fn path_at(&self, n: u64) -> PathBuf {
        self.path.join("kvs.data.".to_owned() + &n.to_string())
    }

    fn active_path(&self, writer: &KvStoreWriter) -> PathBuf {
        self.path_at(writer.active_file)
    }

    // rewrite records to the active file
    fn compact(&self, writer: &mut KvStoreWriter) -> Result<()> {
        let base_file = writer.base_file;
        let active_file = writer.active_file;

        // if kvs.data.0 is used, use active file directly
        // else write keys and values to kvs.data.0
        let target_file = if base_file == 0 { active_file } else { 0 };

        let reader = self.get_reader();
        let mut new_index = HashMap::new();
        for (key, (n, pos)) in &reader.0 {
            // in compact, n < active_file
            let command = KvStore::read_command_from(self.path_at(*n), *pos)?;
            let pos = KvStore::write_command_to_writer(&mut writer.active_writer, &command)?;
            new_index.insert(key.clone(), (target_file, pos));
        }
        writer.active_writer.flush()?;

        // copy active file to kvs.data.0 if kvs.data.0 is removed
        if base_file != 0 {
            copy(self.active_path(writer), self.path_at(target_file))?;
        }

        // swap and drop the index in reader
        self.swap_index(new_index);

        // remove old files (safe by file-rc in OS)
        // TODO: When other reads occur after getting index,
        // it may cause old files to be deleted before reading from them
        if base_file == 0 {
            for i in 0..active_file {
                fs::remove_file(self.path_at(i))?;
            }
        } else {
            for i in base_file..=active_file {
                fs::remove_file(self.path_at(i))?;
            }
        }

        // update fields in writer
        writer.base_file = target_file;
        writer.active_file = target_file;
        writer.active_writer = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(self.active_path(writer))?,
        );
        writer.unused = 0;

        Ok(())
    }

    fn try_compact(&self, last_pos: u64, writer: &mut KvStoreWriter) -> Result<()> {
        if last_pos > SINGLE_FILE_SIZE {
            // create new file if the active file is large
            writer.active_file += 1;
            writer.active_writer = BufWriter::new(
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(self.active_path(writer))?,
            );

            if writer.unused > UNUSED_LIMIT {
                self.compact(writer)?;
            }
        }
        Ok(())
    }

    fn read_command_from(path: PathBuf, pos: u64) -> Result<Command> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        KvStore::read_command_from_reader(&mut reader, pos)
    }

    fn read_command_from_reader(reader: &mut BufReader<File>, pos: u64) -> Result<Command> {
        reader.seek(SeekFrom::Start(pos))?;

        let mut command = Vec::new();
        reader.read_until(b'#', &mut command)?;
        command.pop();

        Ok(serde_json::from_slice(&command)?)
    }

    fn write_command_to_writer(writer: &mut BufWriter<File>, command: &Command) -> Result<u64> {
        writer.seek(SeekFrom::End(0))?;
        let pos = writer.stream_position()?;

        serde_json::to_writer(&mut *writer, command)?;
        writer.write_all(b"#")?;

        Ok(pos)
    }
}

impl Clone for KvStore {
    fn clone(&self) -> Self {
        KvStore {
            reader: Arc::clone(&self.reader),
            writer: Arc::clone(&self.writer),
            path: self.path.clone(),
        }
    }
}

impl KvsEngine for KvStore {
    fn open(path: impl Into<PathBuf>) -> Result<Self> {
        KvStore::open(path)
    }

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
    fn set(&self, key: String, value: String) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();

        let command = Command::Set {
            key: key.clone(),
            value,
        };
        let pos = KvStore::write_command_to_writer(&mut writer.active_writer, &command)?;
        writer.active_writer.flush()?;

        let active_file = writer.active_file;
        let mut new_index = self.get_reader().0.clone();
        if new_index.insert(key, (active_file, pos)).is_some() {
            writer.unused += 1;
        }

        self.swap_index(new_index);
        self.try_compact(pos, &mut writer)
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
    fn get(&self, key: String) -> Result<Option<String>> {
        if let Some(&(n, pos)) = self.get_reader().0.get(&key) {
            match KvStore::read_command_from(self.path_at(n), pos)? {
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
    fn remove(&self, key: String) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();

        let reader = self.get_reader();
        if reader.0.contains_key(&key) {
            let mut new_index = reader.0.clone();
            new_index.remove(&key);

            let command = Command::Rm { key };
            let pos = KvStore::write_command_to_writer(&mut writer.active_writer, &command)?;
            writer.active_writer.flush()?;
            writer.unused += 1;

            self.swap_index(new_index);
            self.try_compact(pos, &mut writer)
        } else {
            Err(Error::KeyNotFound)
        }
    }
}
