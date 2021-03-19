use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
use std::rc::Rc;

use crate::builtin::add_builtin_func;
use crate::chunk::Chunk;
use crate::func::Closure;
use crate::instruction::Instruction;
use crate::stack::Stack;
use crate::state_option::Options;
use crate::value::Value;
use crate::Reader;
use std::path::Path;

const GLOBAL_MAP_INDEX: &'static Value = &Value::Nil;

pub struct State {
    pub(in crate) depth: usize,
    pub(in crate) options: Options,
    pub(in crate) chain: LinkedList<Stack>, // call stack
    registry: HashMap<Value, Value>,
}

fn new_registry_whith_builtin() -> HashMap<Value, Value> {
    let mut global_map = HashMap::new();
    add_builtin_func(&mut global_map);

    let mut registry = HashMap::new();
    registry.insert(
        GLOBAL_MAP_INDEX.clone(),
        Value::Map(RefCell::new(global_map)),
    );

    registry
}

impl State {
    /// create new State using a default stack
    pub fn new() -> State {
        Self::from_stack(Stack::new(20))
    }

    /// create new State using given stack
    pub fn from_stack(stack: Stack) -> State {
        let mut chain = LinkedList::new();
        chain.push_back(stack);
        State {
            depth: 0,
            chain,
            registry: new_registry_whith_builtin(),
            options: Options::default(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> State {
        Self::from_chunk(Reader::from_file(path).into_chunk())
    }

    // create new State using a default stack and load chunk into stack
    pub fn from_chunk(ch: Chunk) -> State {
        let mut state = Self::new();
        let mut func = Closure::with_proto(Rc::new(ch.prototype));
        if func.upval.len() > 0 {
            let gmap = state.registry.get(GLOBAL_MAP_INDEX).unwrap();
            func.upval[0] = Rc::from(RefCell::new(gmap.clone()));
        }

        state.push_value(Value::Function(func));
        state
    }

    pub fn with_option(mut self, opts: Options) -> Self {
        self.options = opts;
        self
    }

    pub(in crate) fn stack(&self) -> &Stack {
        self.chain.front().unwrap()
    }

    pub(in crate) fn stack_mut(&mut self) -> &mut Stack {
        self.chain.front_mut().unwrap()
    }
}

impl State {
    pub fn pc(&self) -> usize {
        self.stack().pc()
    }

    pub fn top(&self) -> usize {
        self.stack().top
    }

    pub fn add_pc(&mut self, n: i32) {
        self.stack_mut().add_pc(n)
    }

    pub fn add_depth(&mut self) {
        self.depth += 1;
    }

    pub fn sub_depth(&mut self) {
        self.depth -= 1;
    }

    pub fn fetch(&mut self) -> Instruction {
        let stack = self.stack_mut();
        let ins = stack.fetch();
        stack.add_pc(1);
        ins
    }

    pub fn abs_index(&self, index: i32) -> usize {
        self.stack().abx_index(index)
    }

    pub fn check_stack(&mut self, n: usize) {
        self.stack_mut().check(n);
    }

    pub fn reg_count(&mut self) -> i32 {
        self.stack().func.max_stack_size as i32
    }

    pub fn pop(&mut self, n: usize) {
        for _ in 0..n {
            self.stack_mut().pop();
        }
    }

    pub fn pop_value(&mut self) -> Value {
        self.stack_mut().pop()
    }

    pub fn push_index(&mut self, index: i32) {
        let stack = self.stack_mut();
        let val = stack.get(index).clone();
        stack.push(val);
    }

    pub fn push_value(&mut self, val: Value) {
        self.stack_mut().push(val);
    }
}

impl State {
    pub fn copy(&mut self, from: i32, to: i32) {
        let stack = self.stack_mut();
        let val = stack.get(from).clone();
        stack.set(to, val);
    }

    /// push value from constant table at index
    pub fn get_const(&mut self, index: usize) {
        let const_val = self.stack().func.constants[index].clone();
        self.push_value(const_val);
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

    pub fn push_global_map(&mut self) {
        let val = self.registry.get(GLOBAL_MAP_INDEX).unwrap().clone();
        assert!(matches!(val, Value::Map(_)));
        self.push_value(val);
    }

    pub fn global_map_get(&mut self, name: String) {
        let gmap = self.registry.get(GLOBAL_MAP_INDEX).unwrap().clone();
        if let Value::Map(mut m) = gmap {
            let val = m.get_mut().get(&Value::String(name)).unwrap();
            self.push_value(val.clone());
        } else {
            panic!("global map is nil")
        }
    }

    pub fn global_map_set(&mut self, name: String) {
        let gmap = self.registry.get(GLOBAL_MAP_INDEX).unwrap();
        assert!(matches!(gmap, &Value::Map(_)));
        if let Value::Map(mut m) = gmap.clone() {
            let val = self.pop_value();
            m.get_mut().insert(Value::String(name), val);
        } else {
            panic!("global map is nil")
        }
    }

    pub fn register(&mut self, name: String, val: Value) {
        self.push_value(val);
        self.global_map_set(name);
    }

    /// pop value set to index
    pub fn replace(&mut self, index: i32) {
        let stack = self.stack_mut();
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
        let stack = self.stack_mut();
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
        self.stack().get(index).into_boolean()
    }

    pub fn to_number(&self, index: i32) -> f64 {
        self.stack().get(index).into_float().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::stack::Stack;
    use crate::state::State;
    use crate::value::Value;

    fn new_state() -> State {
        State::from_stack(Stack::new(20))
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
