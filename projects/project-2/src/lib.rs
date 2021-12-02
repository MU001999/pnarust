//! # kvs
//!
//! `kvs` is a key-value store

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
    index: HashMap<String, u64>,
    file: File,
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
        let mut writer = BufWriter::new(&mut self.file);

        let pos = writer.stream_position()?;
        serde_json::to_writer(
            &mut writer,
            &Command::Set {
                key: key.clone(),
                value,
            },
        )?;
        writer.write_all("#".as_bytes())?;

        self.index.insert(key, pos);
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
        let mut reader = BufReader::new(&self.file);

        if let Some(pos) = self.index.get(&key) {
            reader.seek(SeekFrom::Start(*pos))?;

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
            serde_json::to_writer(&mut self.file, &Command::Rm { key: key.clone() })?;
            self.file.write_all("#".as_bytes())?;

            self.index.remove(&key);
            Ok(())
        } else {
            Err(format_err!("Key not found"))
        }
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path: PathBuf = path.into().join("kvs.data");

        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)?;
        let reader = BufReader::new(&file);

        // rebuild the in-memory index
        let mut index = HashMap::new();

        let mut pos: u64 = 0;
        for command in reader.split(b'#') {
            let command = command?;
            let next_pos = pos + command.len() as u64 + 1;

            let command = serde_json::from_slice(&command)?;
            match command {
                Command::Set { key, .. } => {
                    index.insert(key.clone(), pos);
                }
                Command::Rm { key } => {
                    index.remove(&key);
                }
                _ => (),
            }

            pos = next_pos;
        }

        Ok(KvStore { index, file })
    }
}
