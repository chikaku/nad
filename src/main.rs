use ansi_term::Color::{Green, Red};
use std::env::args;
use std::fmt::Debug;

use rain::Reader;
use rain::State;

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
                Reader::from_file(path).dump_proto();
            })
        }

        if self.exec {
            self.path.iter().for_each(|path| {
                let ch = Reader::from_file(path).into_chunk();
                let mut state = State::from_chunk(ch);

                state.call(0, 0);
            })
        }
    }
}

#[cfg(test)]
mod playground {
    use std::cell::{Ref, RefCell};
    use std::rc::Rc;

    struct Foo<T: Clone> {
        bar: Vec<Rc<RefCell<T>>>,
    }

    impl<T: Clone> Foo<T> {
        fn get_bar(&self, index: usize) -> Ref<T> {
            self.bar[index].borrow()
        }

        fn get_bar_ref(&self) {
            let _: &T = &*self.get_bar(0);
        }
    }

    #[test]
    fn test_ref() {
        let foo = Foo {
            bar: vec![Rc::new(RefCell::new(1))],
        };
        foo.get_bar_ref();
    }

    #[test]
    fn main() {}
}
