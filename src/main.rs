use std::env::args;

mod builtin;
mod chunk;
mod func;
mod instruction;
mod opcode;
mod prototype;
mod reader;
mod stack;
mod state;
mod value;

use crate::state::State;

use ansi_term::Color::{Green, Red};
use std::fmt::Debug;

fn main() {
    let args = args();
    if args.len() < 2 {
        println!("{}: {}", Red.paint("error"), "no input file");
        return;
    }

    let mut ops = Option::default();
    for arg in args.skip(1) {
        match arg.as_str() {
            "-dump" => ops.dump = true,
            "-exec" => ops.exec = true,
            v => ops.add_path(v.to_string()),
        };
    }

    ops.run();
}

#[derive(Default, Debug)]
struct Option {
    dump: bool,
    exec: bool,
    path: Vec<String>,
}

impl Option {
    fn add_path(&mut self, path: String) {
        self.path.push(path)
    }

    fn run(&self) {
        if self.dump {
            self.path.iter().for_each(|path| {
                println!("{}", Green.bold().paint(path));
                reader::Reader::from_file(path).dump_proto();
            })
        }

        if self.exec {
            self.path.iter().for_each(|path| {
                let ch = reader::Reader::from_file(path).into_chunk();
                let mut state = State::from_chunk(ch);

                state.call(0, 0);
            })
        }
    }
}

#[cfg(test)]
mod playground {
    use crate::value::Value;
    use std::cell::{Cell, Ref, RefCell};
    use std::rc::Rc;

    struct Foo<T: Clone> {
        bar: Vec<Rc<RefCell<T>>>,
    }

    impl<T: Clone> Foo<T> {
        fn get_bar(&self, index: usize) -> T {
            self.bar[index].borrow_mut().clone()
        }
    }

    #[test]
    fn main() {
        let size = 2;
        let v = (0..size)
            .into_iter()
            .map(|_| RefCell::from(Value::Nil))
            .collect::<Vec<_>>();

        *v[0].borrow_mut() = Value::String("123".to_string());
        println!("{:?}", v);
    }
}
