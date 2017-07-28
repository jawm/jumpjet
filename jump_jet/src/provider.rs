extern crate byteorder;
extern crate leb;
use self::leb::signed;
use self::leb::unsigned;
use self::byteorder::ByteOrder;
use self::byteorder::LittleEndian;
use self::byteorder::BigEndian;
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
#[derive(Debug)]
pub struct Section {
    id: u8,
    len: u64
}


// -- TYPES
#[derive(Debug)]
pub struct TypesSection {
    section: Section,
    count: u64,
    entries: Vec<TypesSectionEntry>
}
#[derive(Debug)]
pub struct TypesSectionEntry {
    form: i64,
    param_count: u64,
    params: Vec<u64>,
    return_count: u64,
    return_type: Vec<i64>
}
// -- END TYPES


fn read_varuint(size: u32, buffer: &mut Cursor<&[u8]>) -> Result<u64,Error> {
	unsigned(&mut buffer.bytes())
}

fn read_varint(size: u32, buffer: &mut Cursor<&[u8]>) -> Result<i64,Error> {
	signed(&mut buffer.bytes())
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
    		1 => self.read_section_types(Section{len: section_len, id: section_id}, buf),
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

    fn read_section_types(&self, section: Section, buf: &mut Cursor<&[u8]>){
        println!("\t READING TYPES");
    	let count = read_varuint(32, buf).unwrap();
    	println!("count {}", count);

        let mut x_entries: Vec<TypesSectionEntry> = Vec::with_capacity(count as usize);

    	for i in 0..count {
    		let form = read_varint(7, buf).unwrap();
    		assert!(form == 0x60); // form: 'func'
            println!("form {:x}", form);
    		let param_count = read_varuint(32, buf).unwrap();
    		println!("param_count {}", param_count);

            let mut x_params: Vec<u64> = Vec::with_capacity(param_count as usize);

    		for param_index in 0..param_count {
				let param_type = read_varuint(7,buf).unwrap();
				println!("param {} type {:x}", param_index, param_type);

                x_params.push(param_type);
    		}
    		let return_count = read_varuint(1, buf).unwrap();
    		println!("return_count {}", return_count);
    		let return_type = read_varint(7, buf).unwrap();
    		println!("return_type {:x}", return_type);

            x_entries.push(TypesSectionEntry{
                form: form,
                param_count: param_count,
                params: x_params,
                return_count: return_count,
                return_type: vec![return_type]
            });
    	}

        let x_type_section = TypesSection {
            count: count,
            entries: x_entries,
            section: section
        };

        println!("TYPES SECTION");
        println!("{:?}", x_type_section);
    }

    fn read_section_imports(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING IMPORTS {}", len);
    }

    fn read_section_functions(&self, len: u64, buf: &mut Cursor<&[u8]>) {
    	println!("\tREADING FUNCTIONS {}", len);
        let count = read_varuint(32, buf).unwrap();
        println!("count {}", count);
        for i in 0..count {
            let type_index = read_varuint(32, buf).unwrap();
            println!("type_index {}", type_index);
        }
    }

    fn read_section_table(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING TABLE {}", len);
        let count = read_varuint(32, buf).unwrap();
        println!("count {}", count);
        for i in 0..count {
            let elem_type = read_varint(7, buf).unwrap();
            assert!(elem_type == 0x70); // must be anyfunc
            let resizable_limits_flag = read_varuint(1, buf).unwrap();
            let resizable_limits_initial = read_varuint(32, buf).unwrap();
            if resizable_limits_flag == 1 {
                let resizable_limits_maximum = read_varuint(32, buf).unwrap();
                println!("resizable_limits {} {} {}", resizable_limits_flag, resizable_limits_initial, resizable_limits_maximum);
            } else {
                println!("resizable_limits {} {}", resizable_limits_flag, resizable_limits_initial);
            }
        }
    }

    fn read_section_memory(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING MEMORY {}", len);
    }

    fn read_section_global(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING GLOBAL {}", len);
    }

    fn read_section_exports(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING EXPORTS {}", len);
        let count = read_varuint(32, buf).unwrap();
        println!("count {}", count);
        for i in 0..count {
            let field_len = read_varuint(32, buf).unwrap();
            let mut string = vec![];
            buf.take(field_len).read_to_end(&mut string);
            let field_str = String::from_utf8(string).unwrap();
            println!("field_str {}", field_str);

            let external_type = buf.read_u8().unwrap();
            println!("external_type {}", external_type);
            let index = read_varuint(32, buf).unwrap();
            println!("index {}", index);
        }
    }

    fn read_section_start(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING START {}", len);
    }

    fn read_section_elements(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING ELEMENTS {}", len);
        let count = read_varuint(32, buf).unwrap();
        println!("count {}", count);
        for i in 0..count {
            let index = read_varuint(32, buf).unwrap();
            println!("index {}", index);
            
            self.parse_expression(buf);

            // println!("offset {:X}", offset);
            let num_elem = read_varuint(32, buf).unwrap();
            println!("num_elem {}", num_elem);
            for i in 0..num_elem {
                let function_index = read_varuint(32, buf).unwrap();
                println!("function_index {}", function_index);
            }
        }
    }

    fn read_section_code(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING CODE {}", len);
        let count = read_varuint(32, buf).unwrap();
        println!("count {}", count);
        for i in 0..count {
            let body_size = read_varuint(32, buf).unwrap();
            println!("body_size {}", body_size);
            let local_count = read_varuint(32, buf).unwrap();
            println!("local_count {}", local_count);
            for i in 0..local_count {
                let local_type_count = read_varuint(32, buf).unwrap();
                println!("local_type_count {}", local_type_count);
                let local_type = read_varuint(7, buf).unwrap();
                // TODO: parse the type https://github.com/WebAssembly/design/blob/master/BinaryEncoding.md#language-types
            }
            self.parse_expression(buf);
        }
    }

    fn read_section_data(&self, len: u64, buf: &mut Cursor<&[u8]>) {
        println!("\t READING DATA {}", len);
    }

    fn parse_expression(&self, buf: &mut Cursor<&[u8]>) {
        print!("expr ");
        loop {
            let next = buf.read_u8().unwrap();
            print!("{:X} ", next);
            if next == 0x0b {
                break;
            }
        }
        println!("");
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
