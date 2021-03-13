use crate::chunk::Chunk;
use crate::collection::Map;
use crate::instruction::Instruction;
use crate::stack::Stack;
use crate::value::Value;

use std::cell::RefCell;
use std::collections::LinkedList;

pub struct State {
    chain: LinkedList<Stack>, // call stack
}

impl State {
    pub fn new(ch: Chunk) -> State {
        let mut chain = LinkedList::new();
        chain.push_back(Stack::new(0, ch.prototype));
        State { chain }
    }

    pub fn from_stack(stack: Stack) -> State {
        let mut chain = LinkedList::new();
        chain.push_back(stack);
        State { chain }
    }
}

impl State {
    pub fn pc(&self) -> usize {
        self.chain.front().unwrap().pc()
    }

    pub fn add_pc(&mut self, n: i32) {
        self.chain.front_mut().unwrap().add_pc(n)
    }

    pub fn fetch(&mut self) -> Instruction {
        self.chain.front().unwrap().fetch()
    }

    /// push value from constant table at index
    #[allow(mutable_borrow_reservation_conflict)]
    pub fn get_const(&mut self, index: usize) {
        let val = &self.chain.front().unwrap().func.proto.constants[index];
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
        self.chain.front().unwrap().top()
    }

    pub fn abs_index(&self, index: i32) -> usize {
        self.chain.front().unwrap().abx_index(index)
    }

    pub fn check_stack(&mut self, n: usize) {
        self.chain.front_mut().unwrap().check(n);
    }

    pub fn pop(&mut self, n: usize) {
        for _ in 0..n {
            self.chain.front_mut().unwrap().pop();
        }
    }

    pub fn pop_value(&mut self) -> Value {
        self.chain.front_mut().unwrap().pop()
    }

    pub fn copy(&mut self, from: i32, to: i32) {
        let stack = self.chain.front_mut().unwrap();
        let val = stack.get(from).clone();
        stack.set(to, val);
    }

    pub fn push_index(&mut self, index: i32) {
        let stack = self.chain.front_mut().unwrap();
        let val = stack.get(index).clone();
        stack.push(val);
    }

    pub fn push_value(&mut self, val: Value) {
        let stack = self.chain.front_mut().unwrap();
        stack.push(val)
    }

    /// pop value set to index
    pub fn replace(&mut self, index: i32) {
        let stack = self.chain.front_mut().unwrap();
        let val = stack.pop();
        stack.set(index, val);
    }

    pub fn rorate(&mut self, index: i32, n: i32) {
        let high = self.top() - 1;
        let low = self.abs_index(index) - 1;
        let index = if n >= 0 {
            high - (n as usize)
        } else {
            (low as i32 - n - 1) as usize
        };

        let stack = self.chain.front_mut().unwrap();
        stack.reverse(low, index);
        stack.reverse(index + 1, high);
        stack.reverse(low, high);
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

        let stack = self.chain.front_mut().unwrap();
        (n..0).for_each(|_| {
            stack.push(Value::Nil);
        });
        (0..n).for_each(|_| {
            stack.pop();
        });
    }

    pub fn len(&mut self, index: i32) {
        let stack = self.chain.front_mut().unwrap();
        let val = stack.get(index);
        if let Value::String(s) = val {
            let len = s.len() as i64;
            stack.push(Value::Integer(len));
        } else if let Value::Map(m) = val {
            let len = Value::Integer(m.borrow().len() as i64);
            stack.push(len);
        } else {
            panic!("type {} have no length", val.type_name())
        }
    }

    pub fn concat(&mut self, n: usize) {
        let stack = self.chain.front_mut().unwrap();
        match n {
            0 => stack.push(Value::String("".to_string())),
            1 => {}
            n => (1..n).for_each(|_| {
                let s2 = stack.pop().into_string().unwrap();
                let s1 = stack.pop().into_string().unwrap();
                stack.push(Value::String(s1 + s2.as_str()));
            }),
        };
    }

    pub fn compare(&mut self, a: i32, b: i32, op: &'static str) -> bool {
        let stack = self.chain.front().unwrap();
        let a = stack.get(a);
        let b = stack.get(b);
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
        self.chain.front().unwrap().get(index).as_boolean()
    }

    pub fn to_number(&self, index: i32) -> f64 {
        self.chain.front().unwrap().get(index).as_float().unwrap()
    }
}

// Map
impl State {
    pub fn map_new(&mut self, n: usize) {
        self.push_value(Value::Map(RefCell::new(Map::new(n))));
    }

    fn map_get(&mut self, index: i32, key: &Value) {
        let stack = self.chain.front_mut().unwrap();
        if let Value::Map(m) = stack.get(index) {
            let val = m.borrow().get(key).clone();
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
        let key = self.chain.front_mut().unwrap().pop();

        self.map_get(index as i32, &key);
    }

    pub fn map_get_str(&mut self, index: i32, key: String) {
        self.map_get(index, &Value::String(key));
    }

    fn map_set(&mut self, index: usize, key: Value, val: Value) {
        let stack = self.chain.front().unwrap();
        if let Value::Map(m) = stack.get(index as i32) {
            m.borrow_mut().put(key, val);
        } else {
            panic!("not a Map");
        }
    }

    pub fn map_set_top(&mut self, index: i32) {
        let index = self.abs_index(index);
        assert!(index <= self.top() - 2);
        let stack = self.chain.front_mut().unwrap();
        let val = stack.pop();
        let key = stack.pop();
        self.map_set(index, key, val);
    }

    pub fn map_set_idx(&mut self, index: i32, key: i64) {
        let index = self.abs_index(index);
        assert!(index <= self.top() - 2);
        let stack = self.chain.front_mut().unwrap();
        let val = stack.pop();
        let key = Value::Integer(key);
        self.map_set(index, key, val);
    }
}

#[cfg(test)]
mod tests {
    use crate::chunk::Chunk;
    use crate::prototype::Prototype;
    use crate::stack::Stack;
    use crate::state::State;
    use crate::value::Value;

    fn new_state() -> State {
        State::from_stack(Stack::new(20, Prototype::empty()))
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
