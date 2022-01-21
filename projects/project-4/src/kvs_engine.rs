mod kv_store;
mod sled_kvs_engine;

use std::path::PathBuf;

pub use kv_store::KvStore;
pub use sled_kvs_engine::SledKvsEngine;

use crate::Result;

pub trait KvsEngine: Clone + Send + 'static {
    fn open(path: impl Into<PathBuf>) -> Result<Self>;
    fn set(&self, key: String, value: String) -> Result<()>;
    fn get(&self, key: String) -> Result<Option<String>>;
    fn remove(&self, key: String) -> Result<()>;
}
