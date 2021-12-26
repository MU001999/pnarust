use crate::{anyhow, Result, Command, KvsEngine};

pub struct SledKvsEngine {

}

impl KvsEngine for SledKvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        todo!()
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        todo!()
    }

    fn remove(&mut self, key: String) -> Result<()> {
        todo!()
    }
}
