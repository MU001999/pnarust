use crate::{Result, KvsEngine};

pub struct SledKvsEngine {

}

impl KvsEngine for SledKvsEngine {
    fn set(&mut self, _key: String, _value: String) -> Result<()> {
        todo!()
    }

    fn get(&mut self, _key: String) -> Result<Option<String>> {
        todo!()
    }

    fn remove(&mut self, _key: String) -> Result<()> {
        todo!()
    }
}
