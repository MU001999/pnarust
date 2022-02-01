mod kv_store;
mod sled_kvs_engine;

pub use kv_store::KvStore;
pub use sled_kvs_engine::SledKvsEngine;

use crate::Result;

/// A trait for persistent store engines,
/// which provides methods `open`, `set`, `get` and `remove`.
pub trait KvsEngine {
    /// Sets the value of a string key to a string.
    fn set(&mut self, key: String, value: String) -> Result<()>;

    /// Gets the string value of a given string key.
    fn get(&mut self, key: String) -> Result<Option<String>>;

    /// Removes a given string key.
    fn remove(&mut self, key: String) -> Result<()>;
}
