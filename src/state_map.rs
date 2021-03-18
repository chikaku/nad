use std::cell::RefCell;
use std::collections::HashMap;

use crate::value::Value;
use crate::State;

impl State {
    pub fn map_new(&mut self, n: usize) {
        self.push_value(Value::Map(RefCell::new(HashMap::with_capacity(n))));
    }

    fn map_get(&mut self, index: i32, key: &Value) {
        let stack = self.stack_mut();
        if let Value::Map(m) = stack.get(index) {
            let val = m.borrow().get(key).unwrap().clone();
            stack.push(val);
        } else {
            panic!("not a Map");
        }
    }

    pub fn map_get_top(&mut self, index: i32) {
        // `immutable borrow` must occured after `mutable borrow`
        // so pop key first then get map from stack at index
        // we must get absolute index first, because pop will change negative index
        let index = self.abs_index(index);
        let key = self.stack_mut().pop();

        self.map_get(index as i32, &key);
    }

    pub fn map_get_str(&mut self, index: i32, key: String) {
        self.map_get(index, &Value::String(key));
    }

    fn map_set(&mut self, index: usize, key: Value, val: Value) {
        let stack = self.stack_mut();
        if let Value::Map(m) = stack.get(index as i32) {
            m.borrow_mut().insert(key, val);
            stack.set(index as i32, Value::Map(m));
        } else {
            panic!("not a Map");
        }
    }

    pub fn map_set_top(&mut self, index: i32) {
        let index = self.abs_index(index);
        assert!(index <= self.top() - 2);
        let stack = self.stack_mut();
        let val = stack.pop();
        let key = stack.pop();
        self.map_set(index, key, val);
    }

    pub fn map_set_idx(&mut self, index: i32, key: i64) {
        let index = self.abs_index(index);
        assert!(index <= self.top() - 2);
        let stack = self.stack_mut();
        let val = stack.pop();
        let key = Value::Integer(key);
        self.map_set(index, key, val);
    }
}
