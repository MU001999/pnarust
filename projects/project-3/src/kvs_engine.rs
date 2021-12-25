mod kv_store;
mod sled_kvs_engine;

pub use kv_store::KvStore;
pub use sled_kvs_engine::SledKvsEngine;

use crate::Result;

pub trait KvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn get(&mut self, key: String) -> Result<Option<String>>;
    fn remove(&mut self, key: String) -> Result<()>;
}
