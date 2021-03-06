use ansi_term::Color::Green;
use std::rc::Rc;

use crate::instruction::Instruction;
use crate::value::{LocalValue, Upvalue, Value};

#[derive(Default, Hash)]
pub struct Prototype {
    pub source: String,
    pub def_start_line: u32,
    pub def_last_line: u32,
    pub num_params: u8,
    pub is_vararg: u8,
    pub max_stack_size: u8,
    pub code: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub upvalue: Vec<Upvalue>,
    pub protos: Vec<Rc<Prototype>>,
    pub code_line: Vec<u32>,
    pub local_vars: Vec<LocalValue>,
    pub upvalue_name: Vec<String>,
}

impl Prototype {
    pub fn empty() -> Self {
        Self::default()
    }

    fn print_header(&self) {
        println!(
            "{} <{}:{},{}> ({} instruction)",
            if self.def_start_line > 0 {
                "function"
            } else {
                "main"
            },
            self.source,
            self.def_start_line,
            self.def_last_line,
            self.code.len(),
        );

        println!(
            "{}{} params, {} slots, {} upvalues, {} locals, {} constants, {} functions",
            self.num_params,
            if self.is_vararg > 0 { "+" } else { "" },
            self.max_stack_size,
            self.upvalue.len(),
            self.local_vars.len(),
            self.constants.len(),
            self.protos.len(),
        );
    }

    fn print_code(&self) {
        for (index, code) in self.code.iter().enumerate() {
            let line = match self.code_line.get(index) {
                Some(n) => n.to_string(),
                None => String::from("-"),
            };
            println!(
                "\t{}\t[{}]\t{}",
                index + 1,
                line,
                Green.paint(format!("{}", code))
            );
        }
    }

    fn print_consts(&self) {
        println!("Constants ({}):", self.constants.len());
        for (index, value) in self.constants.iter().enumerate() {
            println!("\t{}\t{}", index + 1, Green.paint(format!("{}", value)));
        }
    }

    fn print_local_vars(&self) {
        println!("Locals: ({}):", self.local_vars.len());
        for (index, value) in self.local_vars.iter().enumerate() {
            println!(
                "\t{}\t{}\t{}\t{}",
                index, value.name, value.pc_start, value.pc_end
            )
        }
    }

    fn print_upvalues(&self) {
        println!("Upvalues ({}):", self.upvalue.len());
        for (index, value) in self.upvalue.iter().enumerate() {
            println!(
                "\t{}\t{}\t{}\t{}",
                index,
                self.upvalue_name.get(index).unwrap_or(&"-".to_string()),
                value.in_stack,
                value.idx,
            )
        }
    }

    pub fn dump(&self) {
        self.print_header();
        self.print_code();
        self.print_consts();
        self.print_local_vars();
        self.print_upvalues();
        for proto in &self.protos {
            println!();
            proto.dump();
        }
    }
}
