//! # kvs
//!
//! `kvs` is a key-value store

use std::collections::HashMap;

/// The mainly struct
#[derive(Default)]
pub struct KvStore {
    data: HashMap<String, String>,
}

impl KvStore {
    /// New a KvStore
    pub fn new() -> KvStore {
        KvStore {
            data: HashMap::new(),
        }
    }

    /// Set the given value with the given key.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = kvs::KvStore::new();
    /// store.set("k".to_owned(), "v".to_owned());
    /// assert_eq!(store.get("k".to_owned()), Some("v".to_owned()));
    /// ```
    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
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
    pub fn get(&self, key: String) -> Option<String> {
        self.data.get(&key).cloned()
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
    pub fn remove(&mut self, key: String) {
        self.data.remove(&key);
    }
}
