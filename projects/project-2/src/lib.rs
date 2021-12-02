//! # kvs
//!
//! `kvs` is a key-value store

use structopt::StructOpt;
use std::{collections::HashMap, fs::{File, OpenOptions}, io::BufReader, path::PathBuf};
use serde::{Serialize, Deserialize};

pub type Result<T> = core::result::Result<T, failure::Error>;

#[derive(StructOpt)]
#[derive(Serialize, Deserialize)]
pub enum Command {
    Set {
        key: String,
        value: String,
    },
    Get {
        key: String,
    },
    Rm {
        key: String,
    },
}

/// The mainly struct
pub struct KvStore {
    data: HashMap<String, String>,
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
        serde_json::to_writer(&mut self.file,
            &Command::Set{ key: key.clone(), value: value.clone()}
        )?;
        self.data.insert(key, value);
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
    pub fn get(&self, key: String) -> Result<Option<String>> {
        Ok(self.data.get(&key).cloned())
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
        serde_json::to_writer(&mut self.file,
            &Command::Rm{ key: key.clone()}
        )?;
        self.data.remove(&key);
        Ok(())
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path: PathBuf = path.into().join("kvs.data");

        let file = OpenOptions::new()
            .read(true).append(true).create(true)
            .open(path)?;
        let mut reader = BufReader::new(&file);

        // rebuild the in-memory map
        let mut data = HashMap::new();

        let stream = serde_json::Deserializer::from_reader(&mut reader).into_iter();
        for command in stream {
            let command = command?;
            //let command = serde_json::from_reader(line.as_bytes())?;
            match command {
                Command::Set { key, value } => {
                    data.insert(key.clone(), value.clone());
                },
                Command::Rm { key } => {
                    data.remove(&key);
                },
                _ => continue,
            }
        }

        Ok(KvStore { data, file })
    }
}
