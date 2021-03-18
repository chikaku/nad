use std::rc::Rc;

use crate::func::{Closure, Func};
use crate::instruction::Instruction;
use crate::prototype::Prototype;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct Stack {
    pc: usize,
    pub top: usize,
    pub slots: Vec<Rc<RefCell<Value>>>,
    pub varargs: Rc<Vec<Value>>,
    pub func: Rc<Prototype>,
    pub upvals: Vec<Rc<RefCell<Value>>>,
    pub openuv: HashMap<i32, Rc<RefCell<Value>>>,
}

impl Stack {
    pub fn new(size: usize) -> Stack {
        Stack {
            top: 0,
            pc: 0,
            func: Rc::new(Prototype::empty()),
            varargs: Rc::new(vec![]),
            upvals: vec![],
            openuv: HashMap::new(),
            slots: (0..size)
                .into_iter()
                .map(|_| Rc::new(RefCell::from(Value::Nil)))
                .collect(),
        }
    }

    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn add_pc(&mut self, n: i32) {
        assert!(self.pc as i32 + n >= 0);
        self.pc = (self.pc as i32 + n) as usize;
    }

    pub fn fetch(&mut self) -> Instruction {
        self.func.code[self.pc]
    }

    pub fn check(&mut self, n: usize) {
        let free = self.slots.len() - self.top;
        for _ in free..n {
            self.slots.push(Rc::from(RefCell::from(Value::Nil)))
        }
    }

    pub fn push(&mut self, v: Value) {
        assert!(self.slots.len() > self.top);
        self.slots[self.top] = Rc::from(RefCell::from(v));
        self.top += 1;
    }

    pub fn pushn(&mut self, vs: &Vec<Value>, n: i32) {
        let n = if n < 0 { vs.len() } else { n as usize };
        (0..n).for_each(|index| self.push(vs.get(index).unwrap_or(&Value::Nil).clone()))
    }

    pub fn pop(&mut self) -> Value {
        assert!(self.top > 0);
        self.top -= 1;
        let val = self.slots.remove(self.top);
        self.slots
            .insert(self.top, Rc::from(RefCell::from(Value::Nil)));
        // TODO: how to get value directly
        val.replace(Value::Nil)
    }

    pub fn popn(&mut self, n: usize) -> Vec<Value> {
        let mut val = vec![Value::Nil; n];
        (0..n).for_each(|index| val[n - 1 - index] = self.pop());
        val
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.slots.swap(a, b);
    }

    pub fn reverse(&mut self, mut low: usize, mut high: usize) {
        while low < high {
            self.swap(low, high);
            low += 1;
            high -= 1;
        }
    }

    pub fn abx_index(&self, index: i32) -> usize {
        if index >= 0 {
            index as usize
        } else {
            let index = index + (self.top as i32) + 1;
            assert!(index >= 0, "illegal negative index");
            index as usize
        }
    }

    pub fn is_valid(&self, index: i32) -> bool {
        let index = if index < 0 {
            index + (self.top as i32) + 1
        } else {
            index
        };

        index >= 0 && index <= self.top as i32
    }

    // TODO: return Ref<T> and get ref by &*
    pub fn get(&self, index: i32) -> Value {
        let index = self.abx_index(index);
        assert!(index > 0);

        self.slots
            .get(index - 1)
            .unwrap_or(&Rc::new(RefCell::new(Value::Nil)))
            .borrow_mut()
            .clone()
    }

    pub fn set(&mut self, index: i32, v: Value) {
        let index = self.abx_index(index);
        assert!(0 < index && index <= self.top);
        let source = self.slots[index - 1].clone();
        *source.borrow_mut() = v;
    }
}

#[cfg(test)]
mod tests {
    use crate::stack::Stack;
    use crate::value::Value;

    #[test]
    fn test_stack() {
        let mut s = Stack::new(2);
        assert!(!s.is_valid(1));
        s.push(Value::String("123".to_string()));
        assert!(s.is_valid(1));
        assert_eq!(s.pop(), Value::String("123".to_string()));
        assert!(!s.is_valid(1));

        s.push(Value::Integer(1));
        s.push(Value::Integer(1));
        assert!(s.is_valid(2));
        assert!(s.is_valid(-2));

        s.check(1);
        s.push(Value::Integer(1));
        s.set(1, Value::Integer(2));
        // assert_eq!(s.get(1), &Value::Integer(2));
        //
        // assert_eq!(s.get(4), &Value::_None);
    }

    #[test]
    #[should_panic]
    fn test_push() {
        let mut s = Stack::new(1);
        s.push(Value::Integer(1));
        s.push(Value::Integer(1));
    }
}
