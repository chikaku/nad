use crate::value::Value;
use crate::State;

impl State {
    fn uv_get_index(&mut self, index: i32) -> Value {
        self.stack_mut().upvals[index as usize].borrow_mut().clone()
    }

    fn uv_set_index(&mut self, index: i32, val: Value) {
        let uvref = self.stack_mut().upvals[index as usize].clone();
        *uvref.borrow_mut() = val;
    }

    pub fn uv_get(&mut self, uv_idx: i32, to: i32) {
        let val = self.uv_get_index(uv_idx - 1);
        let stack = self.stack_mut();
        stack.set(to, val);
    }

    pub fn uv_set(&mut self, from: i32, uv_idx: i32) {
        let val = self.stack().get(from);
        self.uv_set_index(uv_idx - 1, val);
    }

    pub fn uv_map_get(&mut self, uv_idx: i32) {
        let uvmap = self.uv_get_index(uv_idx - 1);
        let stack = self.stack_mut();
        let key = stack.pop();
        if let Value::Map(m) = uvmap {
            let val = m.borrow_mut().get(&key).unwrap_or(&Value::Nil).clone();
            stack.push(val);
        } else {
            panic!("not a map");
        }
    }

    pub fn uv_map_set(&mut self, uv_idx: i32) {
        let uvmap = self.uv_get_index(uv_idx - 1);
        let stack = self.stack_mut();
        let val = stack.pop();
        let key = stack.pop();
        if let Value::Map(m) = uvmap {
            m.borrow_mut().insert(key, val);
            self.uv_set_index(uv_idx - 1, Value::Map(m));
        } else {
            panic!("not a map");
        }
    }

    pub fn close_upval(&mut self, _: i32) {
        // ???
    }
}
