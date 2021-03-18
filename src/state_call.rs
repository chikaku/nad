use ansi_term::Color::Green;
use std::rc::Rc;

use crate::func::Closure;
use crate::func::Func;
use crate::stack::Stack;
use crate::value::Value;
use crate::State;

impl State {
    pub fn load_proto(&mut self, index: usize) {
        let stack = self.stack_mut();
        let proto = &stack.func.protos[index];
        let mut closure = Closure::with_proto(proto.clone());
        for (index, uv) in proto.upvalue.iter().enumerate() {
            if uv.in_stack == 1 {
                match stack.openuv.get(&(uv.idx as i32)) {
                    Some(v) => closure.upval[index] = v.clone(),
                    None => {
                        let val = stack.slots[uv.idx as usize].clone();
                        closure.upval[index] = val.clone();
                        stack.openuv.insert(uv.idx as i32, val.clone());
                    }
                }
            } else {
                closure.upval[index] = stack.upvals[uv.idx as usize].clone();
            }
        }

        stack.push(Value::Function(closure));
    }

    pub fn load_vararg(&mut self, n: i32) {
        let stack = self.stack_mut();
        let varargs = stack.varargs.clone();
        stack.check(varargs.len());
        stack.pushn(&varargs, n as i32);
    }

    fn run_function(&mut self) {
        loop {
            let ins = self.fetch();
            if self.options.show_ins {
                println!(
                    "{}{}",
                    "    ".repeat(self.depth - 1),
                    Green.bold().paint(ins.opcode().name)
                );
            }
            ins.exec(self);
            if ins.is_ret() {
                break;
            }
        }
    }

    pub fn call(&mut self, narg: usize, nret: i32) {
        let val = self.stack().get(-(narg as i32 + 1));
        if let Value::Function(f) = val {
            match f.proto {
                Func::Proto(proto) => {
                    let nregs = proto.max_stack_size as usize;
                    let nparams = proto.num_params as i32;
                    let is_vararg = proto.is_vararg == 1;

                    let mut stack = Stack::new(nregs + 20);
                    stack.func = proto.clone();
                    stack.upvals = f.upval;

                    let func_and_args = self.stack_mut().popn(narg + 1);
                    let (params, varargs) = func_and_args.split_at((nparams + 1) as usize);
                    stack.pushn(&params[1..].to_vec(), nparams);
                    stack.top = nregs;

                    if is_vararg && narg > nparams as usize {
                        stack.varargs = Rc::new(varargs.to_vec());
                    }

                    self.chain.push_front(stack);
                    self.add_depth();
                    self.run_function();
                    self.sub_depth();
                    let mut stack = self.chain.pop_front().unwrap();

                    if nret != 0 {
                        let retval = stack.popn(stack.top - nregs);
                        self.stack_mut().check(retval.len());
                        self.stack_mut().pushn(&retval, nret);
                    }
                }
                Func::Builtin(rf) => {
                    let mut stack = Stack::new(narg + 20);
                    stack.upvals = f.upval;
                    let args = self.stack_mut().popn(narg);
                    stack.pushn(&args, narg as i32);
                    self.stack_mut().pop();

                    self.chain.push_front(stack);
                    self.add_depth();
                    let fret = rf(self);
                    self.sub_depth();
                    let mut stack = self.chain.pop_front().unwrap();

                    if nret != 0 {
                        let retval = stack.popn(fret);
                        self.stack_mut().check(retval.len());
                        self.stack_mut().pushn(&retval, nret);
                    }
                }
            }
        } else {
            // TODO: how to avoid this
            panic!("not a function")
        }
    }
}
