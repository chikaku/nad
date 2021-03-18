use ansi_term::Color::Green;
use std::env;

mod util;
use rain::{Options, State};
use util::iter_luac;

#[test]
fn run_code() {
    let opt = Options::new(env::var("DEBUG").is_ok());

    iter_luac(|path| {
        println!("===========================");
        println!("exec: {}", Green.paint(path.to_str().unwrap()));

        let mut state = State::from_file(path);
        state.with_option(opt.clone());
        state.call(0, 0);
    })
}
