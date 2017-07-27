extern crate byteorder;
extern crate leb128;
use self::byteorder::ByteOrder;
use self::byteorder::LittleEndian;
// use self::byteorder::BigEndian;
use std::io::Cursor;
use self::byteorder::ReadBytesExt;

pub trait ProgramProvider {
    fn provide(&self);
}

pub struct BinaryProvider {
    pub buffer: Vec<u8>,
}

fn read_varuint(size: u32, buffer: &mut Cursor<&[u8]>) -> Result<u32,&'static String> {
	// println!("read_varuint {}", size);
	let mut read = 0;

	let interested_bits = 0xEF;
	let continue_bit = 0x80;

	let mut bytes = [0u8; 4];
	while read < size {
		let current = buffer.read_u8().unwrap();
		let value = current & interested_bits;
		bytes[(read%8) as usize] = value;

		// println!("{:b} {} {} ", current, value, value & continue_bit);

		if (value & continue_bit == 0) {
			break;
		}
		read += 8;
	}
	// for byte in &bytes {
	// 	print!("{} ", byte);
	// }
	// println!("");
	Ok(LittleEndian::read_u32(&bytes))
}

fn read_varint(size: u32, buffer: &mut Cursor<&[u8]>) -> Result<i32,&'static String> {
	return Ok(read_varuint(size, buffer).unwrap() as i32);
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
    		3 => self.read_section_functions(section_len, buf),
    		_ => {}
    	}
    }

    fn read_section_types(&self, len: u32, buf: &mut Cursor<&[u8]>){
    	let count = read_varuint(32, buf).unwrap();
    	println!("count {}", count);
    	for i in 0..count {
    		let form = read_varint(7, buf).unwrap();
    		assert!(form == 0x60); // form: 'func'
    		let param_count = read_varuint(32, buf).unwrap();
    		println!("param_count {}", param_count);
    		for param_index in 0..param_count {
				let param_type = read_varuint(7,buf).unwrap();
				println!("param {} type {}", param_index, param_type);
    		}
    		let return_count = read_varuint(1, buf).unwrap();
    		println!("return_count {}", return_count);
    		let return_type = buf.read_u8().unwrap();//read_varint(7, buf).unwrap();
    		println!("return_type {:x}", return_type);
    	}
    	println!("END SECTION");
    }

    fn read_section_functions(&self, len: u32, buf: &mut Cursor<&[u8]>) {
    	println!("reading function declarations");
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
