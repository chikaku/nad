use crate::prototype::Prototype;
use crate::state::State;
use crate::value::Value::Function;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Clone)]
pub enum Func {
    Proto(Rc<Prototype>),
    RS(RSFunc),
}

impl Hash for Func {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Func::Proto(p) => p.hash(state),
            Func::RS(rs) => "rsfunc".hash(state),
        }
    }
}

pub type RSFunc = fn(&mut State) -> usize;

impl Func {
    pub fn new_proto(proto: Rc<Prototype>) -> Self {
        Func::Proto(proto)
    }
}
