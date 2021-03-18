use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub};

use crate::value::IntoError;
use crate::value::IntoResult;
use crate::value::Value;

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
impl_op!(Rem, rem, %);
impl_opf!(Div, div, /);
impl_opb!(BitAnd, bitand, &);
impl_opb!(BitOr, bitor, |);
impl_opb!(BitXor, bitxor, ^);
impl_opb!(Shl, shl, <<);
impl_opb!(Shr, shr, >>);

impl<'m> Neg for Value {
    type Output = IntoResult<Value>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Integer(i) => Ok(Value::Integer(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(IntoError::FloatToInteger),
        }
    }
}

impl<'m> Not for Value {
    type Output = IntoResult<Value>;

    fn not(self) -> Self::Output {
        let v1 = self.into_integer()?;
        Ok(Value::Integer(!v1))
    }
}
