//! # kvs
//!
//! `kvs` is a key-value store

use core::panic;
use std::{collections::HashMap, path::PathBuf};

pub type Result<T> = core::result::Result<T, failure::Error>;

/// The mainly struct
#[derive(Default)]
pub struct KvStore {
    data: HashMap<String, String>,
}

impl KvStore {
    /// Set the given value with the given key.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = kvs::KvStore::new();
    /// store.set("k".to_owned(), "v".to_owned());
    /// assert_eq!(store.get("k".to_owned()), Some("v".to_owned()));
    /// ```
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.data.insert(key, value);
        Ok(())
    }

    /// Get the corresponding value of the given key,
    /// return None if the key not exists.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = kvs::KvStore::new();
    /// assert_eq!(store.get("k".to_owned()), None);
    /// store.set("k".to_owned(), "v".to_owned());
    /// assert_eq!(store.get("k".to_owned()), Some("v".to_owned()));
    /// ```
    pub fn get(&self, key: String) -> Result<Option<String>> {
        Ok(self.data.get(&key).cloned())
    }

    /// Remove the given key and the corresponding value.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = kvs::KvStore::new();
    /// store.set("k".to_owned(), "v".to_owned());
    /// assert_eq!(store.get("k".to_owned()), Some("v".to_owned()));
    /// store.remove("k".to_owned());
    /// assert_eq!(store.get("k".to_owned()), None);
    /// ```
    pub fn remove(&mut self, key: String) -> Result<()> {
        self.data.remove(&key);
        Ok(())
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        panic!("unimplemented!")
    }
}
