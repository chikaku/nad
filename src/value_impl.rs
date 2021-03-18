use crate::value::IntoError;
use crate::value::IntoResult;
use crate::value::Value;

fn float_to_integer(n: f64) -> Result<i64, IntoError> {
    match n == (n as i64) as f64 {
        true => Ok(n as i64),
        false => Err(IntoError::FloatToInteger),
    }
}

#[allow(dead_code)]
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
    pub fn as_integer(&self) -> IntoResult<i64> {
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

    pub fn as_float(&self) -> IntoResult<f64> {
        match self {
            &Value::Float(f) => Ok(f),
            &Value::Integer(v) => Ok(v as f64),
            Value::String(s) => s.parse::<f64>().map_err(IntoError::Float),
            _ => Err(IntoError::TypeUnsupported),
        }
    }

    pub fn into_string(self) -> IntoResult<String> {
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
    use crate::value::Value;
    use crate::value_impl::float_to_integer;

    #[test]
    fn test_float_to_integer() {
        assert!(float_to_integer(f64::MAX).is_err());
        assert_eq!(float_to_integer(i64::MAX as f64).ok(), Some(i64::MAX));

        let v1 = Value::Float(1.0);
        let v2 = Value::Integer(2);
        assert!(v1 < v2);
    }
}
