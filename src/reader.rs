use std::fs::File;
use std::io;
use std::io::Read;
use std::ops::Add;
use std::path::Path;
use std::rc::Rc;

use crate::chunk::{Chunk, Header};
use crate::instruction::Instruction;
use crate::prototype::Prototype;
use crate::value::{LocalValue, Upvalue, Value};

pub struct Reader<T: std::io::Read> {
    r: T,
    file_name: String,
}

impl Reader<io::BufReader<File>> {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Reader<io::BufReader<File>> {
        let f = File::open(&path).unwrap();
        let name = path.as_ref().to_str().unwrap();
        Reader {
            r: io::BufReader::new(f),
            file_name: String::from("@").add(name),
        }
    }
}

impl<'a> Reader<io::BufReader<&'a [u8]>> {
    pub fn from_str<S: AsRef<[u8]> + ?Sized>(s: &'a S) -> Reader<io::BufReader<&'a [u8]>> {
        Reader {
            r: io::BufReader::new(s.as_ref()),
            file_name: "=buffer".to_string(),
        }
    }
}

impl<T: std::io::Read> Reader<T> {
    fn read_byte(&mut self) -> u8 {
        let mut data: [u8; 1] = [0];
        self.r.read_exact(&mut data).unwrap();
        u8::from_le_bytes(data)
    }

    pub fn read_bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut data: [u8; N] = [0; N];
        self.r.read_exact(&mut data).unwrap();
        data
    }

    pub fn read_uint32(&mut self) -> u32 {
        let mut data: [u8; 4] = [0; 4];
        self.r.read_exact(&mut data).unwrap();
        u32::from_le_bytes(data)
    }

    pub fn read_uint64(&mut self) -> u64 {
        let mut data: [u8; 8] = [0; 8];
        self.r.read_exact(&mut data).unwrap();
        u64::from_le_bytes(data)
    }

    pub fn read_luaint(&mut self) -> i64 {
        let mut data: [u8; 8] = [0; 8];
        self.r.read_exact(&mut data).unwrap();
        i64::from_le_bytes(data)
    }

    pub fn read_luanum(&mut self) -> f64 {
        let mut data: [u8; 8] = [0; 8];
        self.r.read_exact(&mut data).unwrap();
        f64::from_le_bytes(data)
    }

    pub fn read_string(&mut self) -> String {
        let mut size = self.read_byte() as u64;
        if size == 0 {
            return String::from("");
        }

        if size == 0xFF {
            size = self.read_uint64()
        }

        let mut buffer = Vec::new();
        self.r
            .by_ref()
            .take(size - 1)
            .read_to_end(&mut buffer)
            .unwrap();
        unsafe { String::from_utf8_unchecked(buffer) }
    }

    pub fn check_header(&mut self) -> Header {
        use crate::chunk::LUAC_HEADER;
        let h = LUAC_HEADER;

        assert_eq!(self.read_bytes(), h.signature, "not a precompiled chunk!");
        assert_eq!(self.read_byte(), h.version, "version mismatch");
        assert_eq!(self.read_byte(), h.format, "format mismatch");
        assert_eq!(self.read_bytes(), h.luac_data, "corrupted!");
        assert_eq!(self.read_byte(), h.cint_size, "int size mismatch");
        assert_eq!(self.read_byte(), h.sizet_size, "size_t size mismatch");
        assert_eq!(self.read_byte(), h.ins_size, "instruction size mismatch");
        assert_eq!(self.read_byte(), h.luaint_size, "lua integer size mismatch");
        assert_eq!(self.read_byte(), h.luanum_size, "lua number size mismatch");
        assert_eq!(self.read_luaint(), h.luac_int, "endianness mismatch");
        assert_eq!(self.read_luanum(), h.luac_num, "float format mismatch");

        h.clone()
    }

    fn read_code(&mut self) -> Vec<Instruction> {
        let count = self.read_uint32();
        let mut code = Vec::with_capacity(count as usize);
        for _ in 0..count {
            code.push(Instruction(self.read_uint32()))
        }
        code
    }

    fn read_constants(&mut self) -> Vec<Value> {
        let count = self.read_uint32();
        let mut consts = Vec::with_capacity(count as usize);
        for _ in 0..count {
            consts.push(self.read_constant())
        }
        consts
    }

    fn read_constant(&mut self) -> Value {
        use crate::value;
        match self.read_byte() {
            value::CONST_TAG_NIL => Value::Nil,
            value::CONST_TAG_BOOL => Value::Bool(self.read_byte() != 0),
            value::CONST_TAG_INT => Value::Integer(self.read_luaint()),
            value::CONST_TAG_NUM => Value::Float(self.read_luanum()),
            value::CONST_TAG_SHORT_STR => Value::String(self.read_string()),
            value::CONST_TAG_LONG_STR => Value::String(self.read_string()),
            _ => panic!("corrupted"),
        }
    }

    fn read_upvalues(&mut self) -> Vec<Upvalue> {
        let count = self.read_uint32();
        let mut upvalues = Vec::with_capacity(count as usize);
        for _ in 0..count {
            upvalues.push(Upvalue {
                in_stack: self.read_byte(),
                idx: self.read_byte(),
            })
        }
        upvalues
    }

    fn read_protos(&mut self, parent_source: &str) -> Vec<Rc<Prototype>> {
        let count = self.read_uint32();
        let mut protos = Vec::with_capacity(count as usize);
        for _ in 0..count {
            protos.push(Rc::new(self.read_prototype(parent_source)))
        }
        protos
    }

    fn read_code_line(&mut self) -> Vec<u32> {
        let count = self.read_uint32();
        let mut code_line = Vec::with_capacity(count as usize);
        for _ in 0..count {
            code_line.push(self.read_uint32())
        }
        code_line
    }

    fn read_local_vars(&mut self) -> Vec<LocalValue> {
        let count = self.read_uint32();
        let mut vars = Vec::with_capacity(count as usize);
        for _ in 0..count {
            vars.push(LocalValue {
                name: self.read_string(),
                pc_start: self.read_uint32(),
                pc_end: self.read_uint32(),
            })
        }
        vars
    }

    fn read_upvalue_name(&mut self) -> Vec<String> {
        let count = self.read_uint32();
        let mut names = Vec::with_capacity(count as usize);
        for _ in 0..count {
            names.push(self.read_string())
        }
        names
    }

    fn read_prototype(&mut self, parent_source: &str) -> Prototype {
        let mut source = self.read_string();
        if source.is_empty() {
            source = parent_source.to_string();
        }

        Prototype {
            source: String::from(source.as_str()),
            def_start_line: self.read_uint32(),
            def_last_line: self.read_uint32(),
            num_params: self.read_byte(),
            is_vararg: self.read_byte(),
            max_stack_size: self.read_byte(),
            code: self.read_code(),
            constants: self.read_constants(),
            upvalue: self.read_upvalues(),
            protos: self.read_protos(source.as_str()),
            code_line: self.read_code_line(),
            local_vars: self.read_local_vars(),
            upvalue_name: self.read_upvalue_name(),
        }
    }

    pub fn prototype(&mut self) -> Prototype {
        self.check_header();
        self.read_byte();
        self.read_prototype(self.file_name.clone().as_str())
    }

    pub fn into_chunk(mut self) -> Chunk {
        Chunk {
            header: self.check_header(),
            upvalue_size: self.read_byte(),
            prototype: self.read_prototype(self.file_name.clone().as_str()),
        }
    }

    pub fn dump_proto(&mut self) {
        self.prototype().dump();
    }
}

#[cfg(test)]
mod tests {
    use crate::reader::Reader;

    #[test]
    fn read_byte() {
        let mut r = Reader::from_str("123");
        assert_eq!(r.read_byte(), '1' as u8);
        assert_eq!(r.read_byte(), '2' as u8);
        assert_eq!(r.read_byte(), '3' as u8);
    }
}
