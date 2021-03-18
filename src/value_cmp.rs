use crate::value::Value;
use std::cmp::Ordering;

impl std::cmp::PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Nil => matches!(other, Value::Nil),
            Value::Bool(a) => match other {
                Value::Bool(b) => a == b,
                _ => false,
            },
            &Value::Integer(i1) => match other {
                &Value::Integer(i2) => i1 == i2,
                &Value::Float(f2) => (i1 as f64) == f2,
                _ => false,
            },
            &Value::Float(f1) => match other {
                &Value::Integer(i2) => f1 == (i2 as f64),
                &Value::Float(f2) => f1 == f2,
                _ => false,
            },
            Value::String(s1) => match other {
                Value::String(s2) => s1 == s2,
                _ => false,
            },
            Value::Map(m1) => match other {
                Value::Map(m2) => m1 == m2,
                _ => false,
            },
            Value::Function(_) => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl std::cmp::PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.eq(other) {
            Some(Ordering::Equal)
        } else if self.le(other) {
            Some(Ordering::Less)
        } else if self.gt(other) {
            Some(Ordering::Greater)
        } else {
            None
        }
    }

    fn lt(&self, other: &Self) -> bool {
        match self {
            &Value::Integer(i1) => match other {
                &Value::Integer(i2) => i1 < i2,
                &Value::Float(f2) => (i1 as f64) < f2,
                _ => panic!("comparison error"),
            },
            &Value::Float(f1) => match other {
                &Value::Float(f2) => f1 < f2,
                &Value::Integer(i2) => f1 < (i2 as f64),
                _ => panic!("comparison error"),
            },
            Value::String(s1) => match other {
                Value::String(s2) => s1 < s2,
                _ => panic!("comparison error"),
            },
            _ => panic!("comparison error"),
        }
    }

    fn le(&self, other: &Self) -> bool {
        self.lt(other) || self.eq(other)
    }

    fn gt(&self, other: &Self) -> bool {
        match self {
            &Value::Integer(i1) => match other {
                &Value::Integer(i2) => i1 > i2,
                &Value::Float(f2) => (i1 as f64) > f2,
                _ => panic!("comparison error"),
            },
            &Value::Float(f1) => match other {
                &Value::Float(f2) => f1 > f2,
                &Value::Integer(i2) => f1 > (i2 as f64),
                _ => panic!("comparison error"),
            },
            Value::String(s1) => match other {
                Value::String(s2) => s1 > s2,
                _ => panic!("comparison error"),
            },
            _ => panic!("comparison error"),
        }
    }

    fn ge(&self, other: &Self) -> bool {
        self.gt(other) || self.eq(other)
    }
}

impl Eq for Value {}
