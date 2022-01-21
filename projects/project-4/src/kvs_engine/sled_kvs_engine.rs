use super::KvsEngine;
use crate::{Error, Result};
use std::path::PathBuf;

#[derive(Clone)]
pub struct SledKvsEngine {
    db: sled::Db,
}

impl SledKvsEngine {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        Ok(SledKvsEngine {
            db: sled::open(path.into())?,
        })
    }
}

impl KvsEngine for SledKvsEngine {
    fn open(path: impl Into<PathBuf>) -> Result<Self> {
        SledKvsEngine::open(path)
    }

    fn set(&self, key: String, value: String) -> Result<()> {
        self.db.insert(key.as_str(), value.as_str())?;
        self.db.flush()?;
        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        let value = self.db.get(key.as_str())?;
        Ok(value.map(|val| String::from_utf8_lossy(val.as_ref()).to_string()))
    }

    fn remove(&self, key: String) -> Result<()> {
        if self.db.contains_key(key.as_str())? {
            self.db.remove(key.as_str())?;
            self.db.flush()?;
            Ok(())
        } else {
            Err(Error::KeyNotFound)
        }
    }
}
