use crate::value::Value;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unimplemented!()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Map {
    hmap: HashMap<Value, Value>,
}

impl Map {
    pub fn new(n: usize) -> Self {
        Map {
            hmap: HashMap::with_capacity(n),
        }
    }

    pub fn get(&self, key: &Value) -> &Value {
        self.hmap.get(key).unwrap()
    }

    pub fn put(&mut self, key: Value, val: Value) {
        self.hmap.insert(key, val).unwrap();
    }

    pub fn len(&self) -> usize {
        self.hmap.len()
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{\n");
        for (key, val) in &self.hmap {
            f.write_str(format!("  {}: {}", key, val).as_str());
        }
        write!(f, "}}")
    }
}
