use crate::prototype::Prototype;
use std::rc::Rc;

pub struct Func {
    pub proto: Rc<Prototype>,
}

impl Func {
    pub fn new(proto: Rc<Prototype>) -> Self {
        Func { proto }
    }
}
