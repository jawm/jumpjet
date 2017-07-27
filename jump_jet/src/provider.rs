extern crate byteorder;
extern crate leb;
use self::leb::signed;
use self::leb::unsigned;
use self::byteorder::ByteOrder;
use self::byteorder::LittleEndian;
// use self::byteorder::BigEndian;
use std::io::Cursor;
use std::io::Error;
use std::io::Read;
use self::byteorder::ReadBytesExt;

pub trait ProgramProvider {
    fn provide(&self);
}

pub struct BinaryProvider {
    pub buffer: Vec<u8>,
}

fn read_varuint(size: u32, buffer: &mut Cursor<&[u8]>) -> Result<u64,Error> {
	unsigned(&mut buffer.bytes())
}

fn read_varint(size: u32, buffer: &mut Cursor<&[u8]>) -> Result<i64,Error> {
	signed(&mut buffer.bytes())
}

fn read_string(buffer: &mut Cursor<&[u8]>) -> (u32, u32) {
	let length = read_varuint(32, buffer).unwrap();
	println!("len {}", length);
	(6,7)
}

impl BinaryProvider {
    pub fn new(buffer: Vec<u8>) -> Self{
        BinaryProvider {buffer: buffer}
    }

    fn read_section(&self, buf: &mut Cursor<&[u8]>){
    	let section_id = buf.read_u8().unwrap();
    	println!("section_id {:x}", section_id);
    	let section_len = read_varuint(7, buf).unwrap();
    	println!("section_len {}", section_len);
    	match section_id {
    		1 => self.read_section_types(section_len, buf),
            2 => self.read_section_imports(section_len, buf),
    		3 => self.read_section_functions(section_len, buf),
            4 => self.read_section_table(section_len, buf),
            5 => self.read_section_memory(section_len, buf),
            6 => self.read_section_global(section_len, buf),
            7 => self.read_section_exports(section_len, buf),
            8 => self.read_section_start(section_len, buf),
            9 => self.read_section_elements(section_len, buf),
            10=> self.read_section_code(section_len, buf),
            11=> self.read_section_data(section_len, buf),
    		_ => panic!("Unknown section id! Exiting")
    	}
        println!("\tEND SECTION");
    }

    fn read_section_types(&self, len: u64, buf: &mut Cursor<&[u8]>){
        println!("\t READING TYPES {}", len);
    	let count = read_varuint(32, buf).unwrap();
    	println!("count {}", count);
    	for i in 0..count {
    		let form = read_varint(7, buf).unwrap();
    		assert!(form == 0x60); // form: 'func'
            println!("form {:x}", form);
    		let param_count = read_varuint(32, buf).unwrap();
    		println!("param_count {}", param_count);
    		for param_index in 0..param_count {
				let param_type = read_varuint(7,buf).unwrap();
				println!("param {} type {:x}", param_index, param_type);
    		}
    		let return_count = read_varuint(1, buf).unwrap();
    		println!("return_count {}", return_count);
    		let return_type = buf.read_u8().unwrap();//read_varint(7, buf).unwrap();
    		println!("return_type {:x}", return_type);
    	}
    }

    fn read_section_imports(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING IMPORTS {}", len);
    }

    fn read_section_functions(&self, len: u64, buf: &mut Cursor<&[u8]>) {
    	println!("\tREADING FUNCTIONS {}", len);
        for i in 0..len {
            buf.read_u8().unwrap();
        }
    }

    fn read_section_table(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING TABLE {}", len);
    }

    fn read_section_memory(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING MEMORY {}", len);
    }

    fn read_section_global(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING GLOBAL {}", len);
    }

    fn read_section_exports(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING EXPORTS {}", len);
    }

    fn read_section_start(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING START {}", len);
    }

    fn read_section_elements(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING ELEMENTS {}", len);
    }

    fn read_section_code(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING CODE {}", len);
    }

    fn read_section_data(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING DATA {}", len);
    }
}

impl ProgramProvider for BinaryProvider {
    fn provide(&self) {
        let mut buf = Cursor::new(&self.buffer[..]);
        let magic_number = buf.read_u32::<LittleEndian>().unwrap();
        let version_number = buf.read_u32::<LittleEndian>().unwrap();
        println!("magic_number {:X}", magic_number);
        println!("version_number {}", version_number);
        while buf.position() < self.buffer.len() as u64 {
        	self.read_section(&mut buf);
        }
    }
}
