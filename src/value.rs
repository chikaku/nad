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

pub enum Constant {
    Nil,
    Bool(bool),
    Integer(i64),
    Number(f64),
    String(String),
}

use std::fmt;

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constant::Nil => write!(f, "nil"),
            Constant::Bool(v) => write!(f, "{}", v),
            Constant::Integer(v) => write!(f, "{}", v),
            Constant::Number(v) => write!(f, "{}", v),
            Constant::String(v) => write!(f, "\"{}\"", v),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    None,
    Nil,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
}
