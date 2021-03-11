use crate::instruction::Instruction;
use crate::prototype::Prototype;
use crate::stack::Stack;
use crate::value::Value;

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
    #[warn(mutable_borrow_reservation_conflict)]
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
            self.stack.push(Value::None);
        });
        (0..n).for_each(|_| {
            self.stack.pop();
        });
    }
}

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
        assert_eq!(state.pop_value(), Value::None);
    }
}
