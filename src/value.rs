use std::fmt;
use std::fmt::Formatter;
use std::num::{ParseFloatError, ParseIntError};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, MulAssign, Neg, Not, Rem, Shl, Shr, Sub};

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

macro_rules! impl_opf {
    ($t:ident, $m:ident, $op:tt) => {
        impl $t for Value {
            type Output = Result<Value, IntoError>;

            fn $m(self, rhs: Self) -> Self::Output {
                let f1 = self.into_float()?;
                let f2 = rhs.into_float()?;
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
                let v1 = self.into_integer()?;
                let v2 = rhs.into_integer()?;
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

                let f1 = self.into_float()?;
                let f2 = rhs.into_float()?;
                Ok(Value::Float(f1 $op f2))
            }
        }
    };
}

impl_op!(Add, add, +);
impl_op!(Sub, sub, -);
impl_op!(Mul, mul, *);
impl_opf!(Div, div, /);
impl_op!(Rem, rem, %);
impl_opb!(BitAnd, bitand, &);
impl_opb!(BitOr, bitor, |);
impl_opb!(BitXor, bitxor, ^);
impl_opb!(Shl, shl, <<);
impl_opb!(Shr, shr, >>);

impl Neg for Value {
    type Output = Result<Value, IntoError>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Integer(i) => Ok(Value::Integer(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(IntoError::FloatToInteger),
        }
    }
}

impl Not for Value {
    type Output = Result<Value, IntoError>;

    fn not(self) -> Self::Output {
        let v1 = self.into_integer()?;
        Ok(Value::Integer(!v1))
    }
}

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

fn float_to_integer(n: f64) -> Result<i64, IntoError> {
    match n == (n as i64) as f64 {
        true => Ok(n as i64),
        false => Err(IntoError::FloatToInteger),
    }
}

impl Value {
    pub fn into_integer(self) -> Result<i64, IntoError> {
        match self {
            Value::Integer(v) => Ok(v),
            Value::Float(f) => float_to_integer(f),
            Value::String(s) => s
                .parse::<f64>()
                .map_err(IntoError::Float)
                .and_then(float_to_integer),
            _ => Err(IntoError::TypeUnsupported),
        }
    }

    pub fn into_float(self) -> Result<f64, IntoError> {
        match self {
            Value::Float(f) => Ok(f),
            Value::Integer(v) => Ok(v as f64),
            Value::String(s) => s.parse::<f64>().map_err(IntoError::Float),
            _ => Err(IntoError::TypeUnsupported),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::value::float_to_integer;

    #[test]
    fn test_float_to_integer() {
        assert!(float_to_integer(f64::MAX).is_err());
        assert_eq!(float_to_integer(i64::MAX as f64).ok(), Some(i64::MAX));
    }
}
