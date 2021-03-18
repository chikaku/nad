mod util;
use rain::Reader;
use util::iter_luac;

#[test]
fn check_header() {
    iter_luac(|path| {
        Reader::from_file(path).check_header();
    })
}

#[test]
fn read_proto() {
    iter_luac(|path| {
        println!("{:?}", path);
        let _ = Reader::from_file(path).prototype();
    });
}
