mod util;

mod run_code {
    use super::util::iter_luac;
    use nad::{Options, State};

    use ansi_term::Color::Green;
    use std::env;

    #[test]
    fn run_code() {
        let opt = Options::new(env::var("DEBUG").is_ok());

        iter_luac(|path| {
            println!("===========================");
            println!("exec: {}", Green.paint(path.to_str().unwrap()));

            State::from_file(path).with_option(opt.clone()).call(0, 0);
        })
    }
}
