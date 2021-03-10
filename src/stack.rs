use crate::value::Value;

struct Stack {
    slots: Vec<Value>,
    top: usize,
}

impl Stack {
    fn new(size: usize) -> Stack {
        let mut slots = vec![];
        for _ in 0..size {
            slots.push(Value::None)
        }
        Stack { slots, top: 0 }
    }

    fn check(&mut self, n: usize) {
        let free = self.slots.len() - self.top;
        for _ in free..n {
            self.slots.push(Value::None)
        }
    }

    fn push(&mut self, v: Value) {
        assert!(self.slots.len() > self.top);
        self.slots[self.top] = v;
        self.top += 1;
    }

    fn pop(&mut self) -> Value {
        assert!(self.top > 0);
        self.top -= 1;
        let val = self.slots.remove(self.top);
        self.slots.insert(self.top, Value::None);
        val
    }

    fn abx_index(&self, index: i32) -> i32 {
        if index >= 0 {
            index
        } else {
            index + (self.top as i32) + 1
        }
    }

    pub fn is_valid(&self, index: i32) -> bool {
        let index = self.abx_index(index);
        0 < index && index <= self.top as i32
    }

    fn get(&self, index: i32) -> &Value {
        let index = self.abx_index(index);
        if 0 < index && index <= self.top as i32 {
            return &self.slots[(index - 1) as usize];
        }
        &Value::None
    }

    fn set(&mut self, index: i32, v: Value) {
        let index = self.abx_index(index);
        assert!(0 < index && index <= self.top as i32);
        self.slots[(index - 1) as usize] = v
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

        assert_eq!(s.get(4), &Value::None);
    }

    #[test]
    #[should_panic]
    fn test_push() {
        let mut s = Stack::new(1);
        s.push(Value::Integer(1));
        s.push(Value::Integer(1));
    }
}
