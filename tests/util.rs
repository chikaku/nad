use std::fs::read_dir;
use std::path::PathBuf;

pub fn iter_luac<F: FnMut(PathBuf)>(f: F) {
    read_dir("tests/bytecode")
        .unwrap()
        .filter_map(|path| {
            let path = path.ok()?.path();
            path.to_str()?.ends_with(".luac").then(move || path)
        })
        .for_each(f)
}
