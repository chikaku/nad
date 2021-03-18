#[derive(Debug, Default, Clone)]
pub struct Options {
    pub show_ins: bool,
}

impl Options {
    pub fn new(show_ins: bool) -> Options {
        Options { show_ins }
    }
}
