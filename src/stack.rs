use std::rc::Rc;

use crate::func::Func;
use crate::instruction::Instruction;
use crate::value::Value;

pub struct Stack {
    pc: usize,
    pub top: usize,
    slots: Vec<Value>,
    pub varargs: Rc<Vec<Value>>,
    pub func: Option<Func>,
}

impl Stack {
    pub fn new(size: usize) -> Stack {
        Stack {
            slots: vec![Value::Nil; size],
            top: 0,
            pc: 0,
            func: None,
            varargs: Rc::new(vec![]),
        }
    }

    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn add_pc(&mut self, n: i32) {
        assert!(self.pc as i32 + n >= 0);
        self.pc = (self.pc as i32 + n) as usize;
    }

    pub fn fetch(&self) -> Instruction {
        let f = self.func.as_ref().unwrap();
        if let Func::Proto(p) = f {
            return p.code[self.pc];
        } else {
            panic!("fetch on non-proto type")
        }
    }

    pub fn check(&mut self, n: usize) {
        let free = self.slots.len() - self.top;
        for _ in free..n {
            self.slots.push(Value::Nil)
        }
    }

    pub fn push(&mut self, v: Value) {
        assert!(self.slots.len() > self.top);
        self.slots[self.top] = v;
        self.top += 1;
    }

    pub fn pushn(&mut self, vs: &Vec<Value>, n: usize) {
        (0..n).for_each(|index| self.push(vs.get(index).unwrap_or(&Value::Nil).clone()))
    }

    pub fn pop(&mut self) -> Value {
        assert!(self.top > 0);
        self.top -= 1;
        let val = self.slots.remove(self.top);
        self.slots.insert(self.top, Value::Nil);
        val
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

    pub fn get(&self, index: i32) -> &Value {
        let index = self.abx_index(index);
        if 0 < index && index <= self.top {
            return &self.slots[index - 1];
        }
        &Value::_None
    }

    pub fn set(&mut self, index: i32, v: Value) {
        let index = self.abx_index(index);
        assert!(0 < index && index <= self.top);
        self.slots[index - 1] = v
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
        assert_eq!(s.get(1), &Value::Integer(2));

        assert_eq!(s.get(4), &Value::_None);
    }

    #[test]
    #[should_panic]
    fn test_push() {
        let mut s = Stack::new(1);
        s.push(Value::Integer(1));
        s.push(Value::Integer(1));
    }
}
