//! # kvs
//!
//! `kvs` is a key-value store

use failure::format_err;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::{File, OpenOptions}, io::{BufReader, Seek, SeekFrom}, path::PathBuf};
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
        let pos = self.file.stream_position()?;
        serde_json::to_writer(
            &mut self.file,
            &Command::Set {
                key: key.clone(),
                value: value.clone(),
            },
        )?;
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
        if let Some(pos) = self.index.get(&key) {
            self.file.seek(SeekFrom::Start(*pos))?;
            let command: Command = serde_json::from_reader(&self.file)?;
            match command {
                Command::Set { key: _ , value } => Ok(Some(value)),
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
            self.index.remove(&key);
            Ok(())
        } else {
            Err(format_err!("Key not found"))
        }
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path: PathBuf = path.into().join("kvs.index");

        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)?;
        let mut reader = BufReader::new(&file);

        // rebuild the in-memory map
        let mut index = HashMap::new();

        let stream = serde_json::Deserializer::from_reader(&mut reader).into_iter();

        let pos = file.stream_position()?;
        for command in stream {
            let command = command?;
            match command {
                Command::Set { key, .. } => {
                    index.insert(key.clone(), pos);
                }
                Command::Rm { key } => {
                    index.remove(&key);
                }
                _ => continue,
            }
        }

        Ok(KvStore { index, file })
    }
}
