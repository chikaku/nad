use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::prototype::Prototype;
use crate::state::State;
use crate::value::Value;
use std::cell::RefCell;

pub type BuiltinFunc = fn(&mut State) -> usize;

struct Closure {
    proto: Func,
    upval: Vec<RefCell<Value>>,
}

#[derive(Clone)]
pub enum Func {
    Proto(Rc<Prototype>),
    Builtin(BuiltinFunc),
}

impl Hash for Func {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Func::Proto(p) => p.hash(state),
            Func::Builtin(_) => "rsfunc".hash(state),
        }
    }
}

impl Func {
    pub fn with_proto(proto: Rc<Prototype>) -> Self {
        Func::Proto(proto)
    }
}

impl Closure {
    pub fn with_proto(proto: Rc<Prototype>) -> Self {
        let uv_count = proto.upvalue.len();
        Closure {
            proto: Func::Proto(proto),
            upval: vec![RefCell::new(Value::Nil); uv_count],
        }
    }
}
