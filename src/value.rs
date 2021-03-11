pub struct Upvalue {
    pub in_stack: u8,
    pub idx: u8,
}

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

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    None,
    Nil,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

use std::fmt;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => unreachable!(),
            Value::Nil => write!(f, "nil"),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Integer(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "\"{}\"", v),
        }
    }
}
