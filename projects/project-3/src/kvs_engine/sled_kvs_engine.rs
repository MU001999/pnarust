use std::path::PathBuf;
use crate::{KvsEngine, Error, Result};

pub struct SledKvsEngine {
    db: sled::Db
}

impl SledKvsEngine {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        Ok(SledKvsEngine {
            db: sled::open(path.into())?
        })
    }
}

impl KvsEngine for SledKvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.db.insert(key.as_str(), value.as_str())?;
        self.db.flush()?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        let value = self.db.get(key.as_str())?;
        Ok(value.map(|val| String::from_utf8_lossy(val.as_ref()).to_string()))
    }

    fn remove(&mut self, key: String) -> Result<()> {
        if self.db.contains_key(key.as_str())? {
            self.db.remove(key.as_str())?;
            self.db.flush()?;
            Ok(())
        } else {
            Err(Error::KeyNotFound)
        }
    }
}
