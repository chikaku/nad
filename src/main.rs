use std::env::args;

mod chunk;
mod collection;
mod func;
mod instruction;
mod opcode;
mod prototype;
mod reader;
mod stack;
mod state;
mod value;

use crate::func::Func;
use crate::state::State;
use crate::value::Value;
use ansi_term::Color::{Green, Red};

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
                state.register("print".to_string(), Value::Function(Func::RS(print)));
                state.call(0, 0);
            })
        }
    }
}

fn print(state: &mut State) -> usize {
    0
}
