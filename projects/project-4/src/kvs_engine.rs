//! A module for store engines.

mod kv_store;
mod sled_kvs_engine;

pub use kv_store::KvStore;
pub use sled_kvs_engine::SledKvsEngine;

use crate::Result;

use std::path::PathBuf;

/// A trait for persistent store engines,
/// which provides methods `open`, `set`, `get` and `remove`.
pub trait KvsEngine: Clone + Send + 'static {
    /// Opens a store engine from the given path
    fn open(path: impl Into<PathBuf>) -> Result<Self>;

    /// Sets the value of a string key to a string.
    fn set(&self, key: String, value: String) -> Result<()>;

    /// Gets the string value of a given string key.
    fn get(&self, key: String) -> Result<Option<String>>;

    /// Removes a given string key.
    fn remove(&self, key: String) -> Result<()>;
}
