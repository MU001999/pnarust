use std::collections::HashMap;

pub struct KvStore {
    data: HashMap<String, String>,
}

impl KvStore {
    pub fn new() -> KvStore {
        KvStore {
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: String) -> Option<String> {
        let value = self.data.get(&key);
        match value {
            None => None,
            Some(value) => Some(value.clone())
        }
    }

    pub fn remove(&mut self, key: String) {
        self.data.remove(&key);
    }
}
