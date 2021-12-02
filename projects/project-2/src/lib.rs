//! # kvs
//!
//! `kvs` is a key-value store

use walkdir::WalkDir;
use failure::format_err;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write},
    path::PathBuf,
};
use structopt::StructOpt;

pub type Result<T> = core::result::Result<T, failure::Error>;

#[derive(StructOpt, Serialize, Deserialize)]
pub enum Command {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String },
}

/// The mainly struct
pub struct KvStore {
    index: HashMap<String, (u64, u64)>,
    path: PathBuf,
    active_nth_file: u64,
}

impl KvStore {
    fn path_at(&self, n: u64) -> PathBuf {
        self.path.join("kvs.data.".to_owned() + &n.to_string())
    }

    fn active_path(&self) -> PathBuf {
        self.path_at(self.active_nth_file)
    }
}

impl KvStore {
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
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let file = OpenOptions::new()
            .append(true).create(true).open(self.active_path())?;
        let mut writer = BufWriter::new(file);
        writer.seek(SeekFrom::End(0))?;

        let pos = writer.stream_position()?;
        serde_json::to_writer(
            &mut writer,
            &Command::Set {
                key: key.clone(),
                value,
            },
        )?;
        writer.write_all("#".as_bytes())?;

        self.index.insert(key, (self.active_nth_file, pos));

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
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(&(n, pos)) = self.index.get(&key) {
            let file = File::open(self.path_at(n))?;
            let mut reader = BufReader::new(file);

            reader.seek(SeekFrom::Start(pos))?;

            let mut command = Vec::new();
            reader.read_until(b'#', &mut command)?;
            command.pop();

            match serde_json::from_slice(&command)? {
                Command::Set { key: _, value } => Ok(Some(value)),
                _ => Err(format_err!("Err Log!!!")),
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
    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let file = OpenOptions::new().append(true).create(true).open(self.active_path())?;
            let mut writer = BufWriter::new(file);
            writer.seek(SeekFrom::End(0))?;

            serde_json::to_writer(&mut writer, &Command::Rm { key: key.clone() })?;
            writer.write_all("#".as_bytes())?;

            self.index.remove(&key);

            Ok(())
        } else {
            Err(format_err!("Key not found"))
        }
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path: PathBuf = path.into();
        let path_at = |n: u64| path.join("kvs.data.".to_owned() + &n.to_string());

        // rebuild the in-memory index
        let mut index = HashMap::new();

        if !path_at(0).exists() {
            File::create(path_at(0))?;
            return Ok(KvStore { index, path, active_nth_file: 0 });
        }

        let mut nfile: u64 = 0;
        for entry in WalkDir::new(&path).min_depth(1).max_depth(1) {
            if entry?.file_name().to_string_lossy().starts_with("kvs.data.") {
                nfile += 1;
            }
        }
        let mut active_nth_file = nfile - 1;

        let mut all_record_cnt = 0;
        for i in 0..nfile {
            let file = File::open(path_at(i))?;
            let reader = BufReader::new(&file);

            let mut record_cnt = 0;
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
                record_cnt += 1;
            }

            all_record_cnt += record_cnt;

            if i == nfile - 1 && record_cnt > 2 {
                if all_record_cnt / index.len() > 2 {
                    // TODO: optimize
                    // rewrite old records to the nth file
                }

                active_nth_file = nfile;
            }
        }

        Ok(KvStore { index, path, active_nth_file })
    }
}
