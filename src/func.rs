use crate::prototype::Prototype;
use crate::state::State;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

pub type BuiltinFunc = fn(&mut State) -> usize;

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
