use std::env::args;

mod chunk;
mod instruction;
mod opcode;
mod prototype;
mod reader;
mod stack;
mod state;
mod value;

use ansi_term::Color::{Green, Red};

fn main() {
    let args = args();
    if args.len() < 2 {
        println!("{}: {}", Red.paint("error"), "no input file");
        return;
    }

    for path in args.skip(1) {
        println!("{}", Green.bold().paint(&path));
        reader::Reader::from_file(path).dump_proto();
    }
}
