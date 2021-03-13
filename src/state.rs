use crate::collection::Map;
use crate::instruction::Instruction;
use crate::prototype::Prototype;
use crate::stack::Stack;
use crate::value::Value;
use std::cell::RefCell;

pub struct State {
    stack: Stack,
    proto: Prototype,
    pc: usize,
}

impl State {
    pub fn new(size: usize, proto: Prototype) -> State {
        State {
            stack: Stack::new(size),
            proto,
            pc: 0,
        }
    }
}

impl State {
    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn add_pc(&mut self, n: i32) {
        self.pc = (self.pc as i32 + n) as usize
    }

    pub fn fetch(&mut self) -> Instruction {
        self.pc += 1;
        self.proto.code[self.pc - 1]
    }

    /// push value from constant table at index
    #[allow(mutable_borrow_reservation_conflict)]
    pub fn get_const(&mut self, index: usize) {
        let val = &self.proto.constants[index];
        self.push_value(val.clone());
    }

    /// push value from stack index or constant table index
    pub fn get_rk(&mut self, index: i32) {
        if index > 0xFF {
            // push constant value
            self.get_const((index & 0xFF) as usize);
        } else {
            // push register value
            self.push_index(index + 1);
        }
    }
}

impl State {
    pub fn top(&self) -> usize {
        self.stack.top()
    }

    pub fn abs_index(&self, index: i32) -> usize {
        self.stack.abx_index(index)
    }

    pub fn check_stack(&mut self, n: usize) {
        self.stack.check(n)
    }

    pub fn pop(&mut self, n: usize) {
        for _ in 0..n {
            self.stack.pop();
        }
    }

    pub fn pop_value(&mut self) -> Value {
        self.stack.pop()
    }

    #[allow(mutable_borrow_reservation_conflict)]
    pub fn copy(&mut self, from: i32, to: i32) {
        let val = self.stack.get(from);
        self.stack.set(to, val.clone());
    }

    #[allow(mutable_borrow_reservation_conflict)]
    pub fn push_index(&mut self, index: i32) {
        let val = self.stack.get(index);
        self.stack.push(val.clone());
    }

    pub fn push_value(&mut self, val: Value) {
        self.stack.push(val)
    }

    /// pop value set to index
    pub fn replace(&mut self, index: i32) {
        let val = self.stack.pop();
        self.stack.set(index, val);
    }

    pub fn rorate(&mut self, index: i32, n: i32) {
        let high = self.top() - 1;
        let low = self.abs_index(index) - 1;
        let index = if n >= 0 {
            high - (n as usize)
        } else {
            (low as i32 - n - 1) as usize
        };

        self.stack.reverse(low, index);
        self.stack.reverse(index + 1, high);
        self.stack.reverse(low, high);
    }

    pub fn insert(&mut self, index: i32) {
        self.rorate(index, 1);
    }

    pub fn remove(&mut self, index: i32) {
        self.rorate(index, -1);
        self.pop(1);
    }

    pub fn set_top(&mut self, index: i32) {
        let top = self.abs_index(index) as i32;
        let n = self.top() as i32 - top;

        (n..0).for_each(|_| {
            self.stack.push(Value::Nil);
        });
        (0..n).for_each(|_| {
            self.stack.pop();
        });
    }

    #[allow(mutable_borrow_reservation_conflict)]
    pub fn len(&mut self, index: i32) {
        let val = self.stack.get(index);
        if let Value::String(s) = val {
            self.stack.push(Value::Integer(s.len() as i64))
        } else if let Value::Map(m) = val {
            let len = Value::Integer(m.borrow().len() as i64);
            self.stack.push(len);
        } else {
            panic!("type {} have no length", val.type_name())
        }
    }

    pub fn concat(&mut self, n: usize) {
        match n {
            0 => self.stack.push(Value::String("".to_string())),
            1 => {}
            n => (1..n).for_each(|_| {
                let s2 = self.stack.pop().into_string().unwrap();
                let s1 = self.stack.pop().into_string().unwrap();
                self.stack.push(Value::String(s1 + s2.as_str()));
            }),
        };
    }

    pub fn compare(&mut self, a: i32, b: i32, op: &'static str) -> bool {
        let a = self.stack.get(a);
        let b = self.stack.get(b);
        match op {
            "==" => a == b,
            ">" => a > b,
            "<" => a < b,
            ">=" => a >= b,
            "<=" => a <= b,
            _ => panic!("unsupported compare operator"),
        }
    }

    pub fn to_boolean(&self, index: i32) -> bool {
        self.stack.get(index).as_boolean()
    }

    pub fn to_number(&self, index: i32) -> f64 {
        self.stack.get(index).as_float().unwrap()
    }
}

// Map
impl State {
    pub fn map_new(&mut self, n: usize) {
        self.push_value(Value::Map(RefCell::new(Map::new(n))));
    }

    fn map_get(&mut self, index: i32, key: &Value) {
        if let Value::Map(m) = self.stack.get(index) {
            let val = m.borrow().get(key).clone();
            self.stack.push(val);
        } else {
            panic!("not a Map");
        }
    }

    pub fn map_get_top(&mut self, index: i32) {
        // `immutable borrow` must occured after `mutable borrow`
        // so pop key first then get map from stack at index
        // we must get absolute index first, because pop will change negative index
        let index = self.abs_index(index);
        let key = self.stack.pop();

        self.map_get(index as i32, &key);
    }

    pub fn map_get_str(&mut self, index: i32, key: String) {
        self.map_get(index, &Value::String(key));
    }

    fn map_set(&mut self, index: usize, key: Value, val: Value) {
        if let Value::Map(m) = self.stack.get(index as i32) {
            m.borrow_mut().put(key, val);
        } else {
            panic!("not a Map");
        }
    }

    pub fn map_set_top(&mut self, index: i32) {
        let index = self.abs_index(index);
        assert!(index <= self.top() - 2);
        let val = self.stack.pop();
        let key = self.stack.pop();
        self.map_set(index, key, val);
    }

    pub fn map_set_idx(&mut self, index: i32, key: i64) {
        let index = self.abs_index(index);
        assert!(index <= self.top() - 2);
        let key = Value::Integer(key);
        let val = self.stack.pop();
        self.map_set(index, key, val);
    }
}

#[cfg(test)]
mod tests {
    use crate::reader::Reader;
    use crate::state::State;
    use crate::value::Value;

    fn new_state() -> State {
        State::new(
            10,
            Reader::from_file("./luacode/hello_world.luac").prototype(),
        )
    }

    #[test]
    fn test_rotate() {
        let mut state = new_state();
        for index in 1..6 {
            state.push_value(Value::Integer(index));
        }

        state.rorate(2, 1);
        assert_eq!(state.pop_value(), Value::Integer(4));
        assert_eq!(state.pop_value(), Value::Integer(3));
        assert_eq!(state.pop_value(), Value::Integer(2));
        assert_eq!(state.pop_value(), Value::Integer(5));
        assert_eq!(state.pop_value(), Value::Integer(1));

        let mut state = new_state();
        for index in 1..6 {
            state.push_value(Value::Integer(index));
        }
        state.rorate(2, -1);
        assert_eq!(state.pop_value(), Value::Integer(2));
        assert_eq!(state.pop_value(), Value::Integer(5));
        assert_eq!(state.pop_value(), Value::Integer(4));
        assert_eq!(state.pop_value(), Value::Integer(3));
        assert_eq!(state.pop_value(), Value::Integer(1));
    }

    #[test]
    fn test_set_top() {
        let mut state = new_state();
        (1..6).for_each(|index| state.push_value(Value::Integer(index)));

        state.set_top(2);
        assert_eq!(state.pop_value(), Value::Integer(2));
        assert_eq!(state.pop_value(), Value::Integer(1));

        let mut state = new_state();
        (1..2).for_each(|index| state.push_value(Value::Integer(index)));
        state.set_top(2);
        assert_eq!(state.top(), 2);
        assert_eq!(state.pop_value(), Value::Nil);
    }
}
