use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::num::ParseFloatError;
use std::rc::Rc;

use crate::func::Closure;

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
pub type MutValue = Rc<RefCell<Value>>;

#[derive(Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Map(Map),
    Function(Closure),
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.type_name().as_bytes());
        match self {
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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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

pub type IntoResult<T> = Result<T, IntoError>;

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
