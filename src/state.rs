use crate::chunk::Chunk;
use crate::instruction::Instruction;
use crate::stack::Stack;
use crate::value::{Map, Value};

use crate::builtin::add_builtin_func;
use crate::func::{BuiltinFunc, Func};
use crate::prototype::Prototype;
use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
use std::rc::Rc;

const GLOBAL_MAP_INDEX: &Value = &Value::_None;

pub struct State {
    chain: LinkedList<Stack>, // call stack
    registry: Map,
}

fn new_registry_whith_builtin() -> Map {
    let mut global_map = HashMap::new();
    add_builtin_func(&mut global_map);

    let mut registry = HashMap::new();
    registry.insert(
        GLOBAL_MAP_INDEX.clone(),
        Value::Map(RefCell::new(global_map)),
    );

    RefCell::new(registry)
}

impl State {
    /// create new State using given stack
    pub fn from_stack(stack: Stack) -> State {
        let mut chain = LinkedList::new();
        chain.push_back(stack);
        State {
            chain,
            registry: new_registry_whith_builtin(),
        }
    }

    /// create new State using a default stack
    pub fn new() -> State {
        Self::from_stack(Stack::new(20))
    }

    // create new State using a default stack and load chunk into stack
    pub fn from_chunk(ch: Chunk) -> State {
        let mut state = Self::new();
        let func = Func::with_proto(Rc::new(ch.prototype));
        state.push_value(Value::Function(func));
        state
    }

    fn stack(&self) -> &Stack {
        self.chain.front().unwrap()
    }

    fn stack_mut(&mut self) -> &mut Stack {
        self.chain.front_mut().unwrap()
    }
}

impl State {
    pub fn pc(&self) -> usize {
        self.stack().pc()
    }

    pub fn add_pc(&mut self, n: i32) {
        self.stack_mut().add_pc(n)
    }

    pub fn fetch(&mut self) -> Instruction {
        let stack = self.stack_mut();
        let ins = stack.fetch();
        stack.add_pc(1);
        ins
    }

    /// push value from constant table at index
    pub fn get_const(&mut self, index: usize) {
        let proto = match &self.stack().func.as_ref().unwrap() {
            Func::Proto(p) => p.clone(),
            _ => panic!("not a proto"),
        };
        let const_val = &proto.constants[index];
        self.push_value(const_val.clone());
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
        self.stack().top
    }

    pub fn abs_index(&self, index: i32) -> usize {
        self.stack().abx_index(index)
    }

    pub fn check_stack(&mut self, n: usize) {
        self.stack_mut().check(n);
    }

    pub fn reg_count(&self) -> i32 {
        if let Func::Proto(p) = self.stack().func.as_ref().unwrap() {
            return p.max_stack_size as i32;
        } else {
            panic!("not a proto type");
        }
    }

    pub fn pop(&mut self, n: usize) {
        for _ in 0..n {
            self.stack_mut().pop();
        }
    }

    pub fn pop_value(&mut self) -> Value {
        self.chain.front_mut().unwrap().pop()
    }

    pub fn copy(&mut self, from: i32, to: i32) {
        let stack = self.stack_mut();
        let val = stack.get(from).clone();
        stack.set(to, val);
    }

    pub fn push_index(&mut self, index: i32) {
        let stack = self.stack_mut();
        let val = stack.get(index).clone();
        stack.push(val);
    }

    pub fn push_value(&mut self, val: Value) {
        self.stack_mut().push(val);
    }

    pub fn push_global_map(&mut self) {
        let val = self
            .registry
            .borrow()
            .get(GLOBAL_MAP_INDEX)
            .unwrap()
            .clone();
        self.push_value(val);
    }

    pub fn global_map_get(&mut self, name: String) {
        let gmap = self
            .registry
            .borrow()
            .get(GLOBAL_MAP_INDEX)
            .unwrap()
            .clone();
        if let Value::Map(mut m) = gmap {
            let val = m.get_mut().get(&Value::String(name)).unwrap();
            self.push_value(val.clone());
        } else {
            panic!("global map is nil")
        }
    }

    pub fn global_map_set(&mut self, name: String) {
        let gmap = self
            .registry
            .borrow()
            .get(GLOBAL_MAP_INDEX)
            .unwrap()
            .clone();
        if let Value::Map(mut m) = gmap {
            let val = self.pop_value();
            m.get_mut().insert(Value::String(name), val);
            self.registry
                .borrow_mut()
                .insert(GLOBAL_MAP_INDEX.clone(), Value::Map(m.clone()));
            println!("{}", m.get_mut().len());
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
        self.push_value(Value::Map(RefCell::new(HashMap::with_capacity(n))));
    }

    fn map_get(&mut self, index: i32, key: &Value) {
        let stack = self.chain.front_mut().unwrap();
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
        let key = self.chain.front_mut().unwrap().pop();

        self.map_get(index as i32, &key);
    }

    pub fn map_get_str(&mut self, index: i32, key: String) {
        self.map_get(index, &Value::String(key));
    }

    fn map_set(&mut self, index: usize, key: Value, val: Value) {
        let stack = self.chain.front().unwrap();
        if let Value::Map(m) = stack.get(index as i32) {
            m.borrow_mut().insert(key, val);
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

impl State {
    pub fn load_proto(&mut self, index: usize) {
        let this_func = self.stack().func.as_ref().unwrap();
        if let Func::Proto(p) = this_func {
            let proto = p.protos[index].clone();
            self.stack_mut()
                .push(Value::Function(Func::with_proto(proto)));
        } else {
            panic!("not a proto type");
        }
    }

    pub fn load_vararg(&mut self, n: i32) {
        let n = if n < 0 {
            self.stack().varargs.len()
        } else {
            n as usize
        };
        let stack = self.stack_mut();
        stack.check(n);
        let varargs = stack.varargs.clone();
        stack.pushn(&varargs, n);
    }
}

// Call
impl State {
    fn run_function(&mut self) {
        loop {
            let ins = self.fetch();
            ins.exec(self);
            if ins.is_ret() {
                break;
            }
        }
    }

    fn call_function(&mut self, narg: usize, nret: usize, f: Rc<Prototype>) {
        let nregs = f.max_stack_size as usize;
        let nparams = f.num_params as usize;
        let is_vararg = f.is_vararg == 1;

        let mut stack = Stack::new(nregs + 20);
        stack.func = Some(Func::Proto(f));

        let func_and_args = self.stack_mut().popn(narg + 1);
        let (params, varargs) = func_and_args.split_at(nparams + 1);
        stack.pushn(&params[1..].to_vec(), nparams);
        stack.top = nregs;

        if is_vararg && narg > nparams {
            stack.varargs = Rc::new(varargs.to_vec());
        }

        self.chain.push_front(stack);
        self.run_function();
        let mut stack = self.chain.pop_front().unwrap();

        if nret != 0 {
            let retval = stack.popn(stack.top - nregs);
            self.stack_mut().check(retval.len());
            self.stack_mut().pushn(&retval, nret);
        }
    }

    fn call_rs_function(&mut self, narg: usize, nret: usize, f: BuiltinFunc) {
        let mut stack = Stack::new(narg + 20);
        stack.func = Some(Func::Builtin(f));

        let args = self.stack_mut().popn(narg);
        stack.pushn(&args, narg);
        self.stack_mut().pop();

        self.chain.push_front(stack);
        let fret = f(self);
        let mut stack = self.chain.pop_front().unwrap();

        if nret != 0 {
            let retval = stack.popn(fret);
            self.stack_mut().check(retval.len());
            self.stack_mut().pushn(&retval, nret);
        }
    }

    pub fn call(&mut self, narg: usize, nret: usize) {
        let val = self.stack().get(-(narg as i32 + 1)).clone();
        if let Value::Function(f) = val {
            match f {
                Func::Proto(p) => self.call_function(narg, nret, p),
                Func::Builtin(f) => self.call_rs_function(narg, nret, f),
            }
        } else {
            panic!("not a function")
        }
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
