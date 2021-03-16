use std::cmp::Ordering;

use std::fmt;
use std::num::ParseFloatError;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub};

use crate::func::Func;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Hash)]
pub struct Upvalue {
    pub in_stack: u8,
    pub idx: u8,
}

#[derive(Hash)]
pub struct LocalValue {
    pub name: String,
    pub pc_start: u32,
    pub pc_end: u32,
}

pub const CONST_TAG_NIL: u8 = 0x00;
pub const CONST_TAG_BOOL: u8 = 0x01;
pub const CONST_TAG_NUM: u8 = 0x03;
pub const CONST_TAG_INT: u8 = 0x13;
pub const CONST_TAG_SHORT_STR: u8 = 0x04;
pub const CONST_TAG_LONG_STR: u8 = 0x14;

pub type Map = RefCell<HashMap<Value, Value>>;

#[derive(Clone)]
pub enum Value {
    _None,
    Nil,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Map(Map),
    Function(Func),
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.type_name().as_bytes());
        match self {
            Value::_None => {}
            Value::Nil => {}
            Value::Bool(v) => v.hash(state),
            Value::Integer(i) => i.hash(state),
            Value::Float(f) => f.to_be_bytes().hash(state),
            Value::String(s) => s.hash(state),
            Value::Map(m) => m.borrow().keys().for_each(|k| k.hash(state)),
            Value::Function(f) => f.hash(state),
        }
    }
}

macro_rules! impl_opf {
    ($t:ident, $m:ident, $op:tt) => {
        impl $t for Value {
            type Output = Result<Value, IntoError>;

            fn $m(self, rhs: Self) -> Self::Output {
                let f1 = self.as_float()?;
                let f2 = rhs.as_float()?;
                Ok(Value::Float(f1 $op f2))
            }
        }
    };
}

macro_rules! impl_opb {
    ($t:ident, $m:ident, $op:tt) => {
        impl $t for Value {
            type Output = Result<Value, IntoError>;

            fn $m(self, rhs: Self) -> Self::Output {
                let v1 = self.as_integer()?;
                let v2 = rhs.as_integer()?;
                Ok(Value::Integer(v1 $op v2))
            }
        }
    };
}

macro_rules! impl_op {
    ($t:ident, $m:ident, $op:tt) => {
        impl $t for Value {
            type Output = Result<Value, IntoError>;

            fn $m(self, rhs: Self) -> Self::Output {
                if let Value::Integer(v1) = self {
                    if let Value::Integer(v2) = rhs {
                        return Ok(Value::Integer(v1 $op v2));
                    }
                }

                let f1 = self.as_float()?;
                let f2 = rhs.as_float()?;
                Ok(Value::Float(f1 $op f2))
            }
        }
    };
}

impl_op!(Add, add, +);
impl_op!(Sub, sub, -);
impl_op!(Mul, mul, *);
impl_op!(Rem, rem, %);
impl_opf!(Div, div, /);
impl_opb!(BitAnd, bitand, &);
impl_opb!(BitOr, bitor, |);
impl_opb!(BitXor, bitxor, ^);
impl_opb!(Shl, shl, <<);
impl_opb!(Shr, shr, >>);

impl<'m> Neg for Value {
    type Output = Result<Value, IntoError>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Integer(i) => Ok(Value::Integer(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(IntoError::FloatToInteger),
        }
    }
}

impl<'m> Not for Value {
    type Output = Result<Value, IntoError>;

    fn not(self) -> Self::Output {
        let v1 = self.as_integer()?;
        Ok(Value::Integer(!v1))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::_None => write!(f, "None"),
            Value::Nil => write!(f, "Nil"),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Integer(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::Map(m) => write!(f, "{:?}", m),
            Value::Function(_) => write!(f, "{}", "Function?"),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::cmp::PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::_None => matches!(other, _None),
            Value::Nil => matches!(other, Nil),
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

pub enum IntoError {
    Float(ParseFloatError),
    FloatToInteger,
    TypeUnsupported,
}

impl fmt::Display for IntoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntoError::Float(e) => write!(f, "{}", e),
            IntoError::FloatToInteger => write!(f, "{}", "float convert to int error"),
            IntoError::TypeUnsupported => write!(f, "{}", "unsupported type conversion"),
        }
    }
}

impl fmt::Debug for IntoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

fn float_to_integer(n: f64) -> Result<i64, IntoError> {
    match n == (n as i64) as f64 {
        true => Ok(n as i64),
        false => Err(IntoError::FloatToInteger),
    }
}

/// floating point byte
/// EEEEEXXX
/// IF (EEEEE == 0) THEN XXX
/// ELSE (1XXX) * 2 ^ (EEEEE - 1)
pub fn int2fb(mut x: i32) -> i32 {
    let mut e = 0;
    if x < 8 {
        return x;
    }

    while x >= (8 << 4) {
        x = (x + 0xF) >> 4;
        e += 4;
    }

    while x >= (8 << 1) {
        x = (x + 1) >> 1;
        e += 1;
    }

    (e + 1) << 3 | (x - 8)
}

/// int2fb reverse
pub fn fb2int(x: i32) -> i32 {
    if x < 8 {
        x
    } else {
        ((x & 7) + 8) << ((x >> 3) - 1)
    }
}

impl Value {
    pub fn as_integer(&self) -> Result<i64, IntoError> {
        match self {
            &Value::Integer(v) => Ok(v),
            &Value::Float(f) => float_to_integer(f),
            Value::String(s) => s
                .parse::<f64>()
                .map_err(IntoError::Float)
                .and_then(float_to_integer),
            _ => Err(IntoError::TypeUnsupported),
        }
    }

    pub fn as_float(&self) -> Result<f64, IntoError> {
        match self {
            &Value::Float(f) => Ok(f),
            &Value::Integer(v) => Ok(v as f64),
            Value::String(s) => s.parse::<f64>().map_err(IntoError::Float),
            _ => Err(IntoError::TypeUnsupported),
        }
    }

    pub fn into_string(self) -> Result<String, IntoError> {
        match self {
            Value::Float(f) => Ok(f.to_string()),
            Value::Integer(i) => Ok(i.to_string()),
            Value::String(s) => Ok(s),
            _ => Err(IntoError::TypeUnsupported),
        }
    }

    pub fn as_boolean(&self) -> bool {
        match self {
            &Value::Nil => false,
            &Value::Bool(v) => v,
            _ => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::_None => "None",
            Value::Nil => "Nil",
            Value::Integer(_) => "Integer",
            Value::Float(_) => "Float",
            Value::String(_) => "String",
            Value::Bool(_) => "Boolean",
            Value::Map(_) => "Map",
            Value::Function(_) => "Function",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::value::{float_to_integer, Value};

    #[test]
    fn test_float_to_integer() {
        assert!(float_to_integer(f64::MAX).is_err());
        assert_eq!(float_to_integer(i64::MAX as f64).ok(), Some(i64::MAX));

        let v1 = Value::Float(1.0);
        let v2 = Value::Integer(2);
        assert!(v1 < v2);
    }
}
