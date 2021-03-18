use ansi_term::Color::{Green, Red};
use std::env::args;
use std::fmt::Debug;

use rain::State;
use rain::{Options, Reader};

fn main() {
    let args = args();
    if args.len() < 2 {
        println!("{}: {}", Red.paint("error"), "no input file");
        return;
    }

    let mut ops = Option::default();
    for arg in args.skip(1) {
        match arg.as_str() {
            "-debug" => ops.debug = true,
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
    debug: bool,
    path: Vec<String>,
}

impl Option {
    fn add_path(&mut self, path: String) {
        self.path.push(path)
    }

    fn iter_file<F: FnMut(&String)>(&self, f: F) {
        self.path.iter().for_each(f)
    }

    fn run(&self) {
        if self.dump {
            self.iter_file(|path| {
                println!("{}", Green.bold().paint(path));
                Reader::from_file(path).dump_proto();
            });
        }

        if self.exec || !self.dump {
            self.iter_file(|path| {
                let mut state = State::from_file(path);
                state.with_option(Options {
                    show_ins: self.debug,
                });
                state.call(0, 0);
            })
        }
    }
}
