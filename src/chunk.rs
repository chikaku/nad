use std::path::Path;

use crate::prototype::Prototype;
use crate::reader::Reader;

pub struct Chunk {
    pub header: Header,
    pub upvalue_size: u8,
    pub prototype: Prototype,
}

impl Chunk {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        Reader::from_file(path).into_chunk()
    }
}

#[derive(PartialEq, Copy, Clone)]
pub struct Header {
    pub signature: [u8; 4],
    pub version: u8,
    pub format: u8,
    pub luac_data: [u8; 6],
    pub cint_size: u8,
    pub sizet_size: u8,
    pub ins_size: u8,
    pub luaint_size: u8,
    pub luanum_size: u8,
    pub luac_int: i64,
    pub luac_num: f64,
}

pub const LUAC_HEADER: Header = Header {
    signature: [0x1b, 0x4c, 0x75, 0x61],
    version: 0x53,
    format: 0,
    luac_data: [0x19, 0x93, 0x0d, 0x0a, 0x1a, 0x0a],
    cint_size: 4,
    sizet_size: 8,
    ins_size: 4,
    luaint_size: 8,
    luanum_size: 8,
    luac_int: 0x5678,
    luac_num: 370.5,
};
