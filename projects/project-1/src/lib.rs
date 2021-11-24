use std::iter::Map;
use std::panic;

pub struct KvStore {
    _data: Map<String, String>,
}

impl KvStore {
    pub fn new() -> KvStore {
        panic!();
    }

    pub fn set(&mut self, _key: String, _value: String) {
        panic!();
    }

    pub fn get(&self, _key: String) -> Option<String> {
        panic!();
    }

    pub fn remove(&mut self, _key: String) {
        panic!();
    }
}
