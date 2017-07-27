use std::io::{Bytes, Error, Read};

const CONTINUE_MASK: u64 = 0x80;
const VALUE_MASK: u64 = 0x7F;

pub fn signed<R: Read>(buffer: &mut Bytes<R>) -> Result<i64, Error> {
	match unsigned(buffer) {
		Ok(val) => Ok(val as i64),
		Err(e) => Err(e)
	}
}

pub fn unsigned<R: Read>(buffer: &mut Bytes<R>) -> Result<u64, Error> {
	let mut result: u64 = 0;
	let mut shift = 0;
	for object in buffer {
		let byte = object.unwrap() as u64;
		result |= (byte & VALUE_MASK) << shift;
		if byte & CONTINUE_MASK == 0 {
			break;
		}
		shift += 7;
	}
	Ok(result)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Cursor;

	fn x(bytes: Vec<u8>) -> Bytes<Cursor<Vec<u8>>> {
		Cursor::new(bytes).bytes()
	}

    #[test]
    fn unsigned_reads_1_32() {
    	let mut buff = x(vec![1]);
    	assert!(unsigned(&mut buff).unwrap() == 1);
    }

    #[test]
    fn unsigned_reads_2_32() {
    	let mut buff = x(vec![2]);
    	assert!(unsigned(&mut buff).unwrap() == 2);
    }

    #[test]
    fn unsigned_reads_500_32() {
    	let mut buff = x(vec![
    		0b1111_0100u8,
    		0b0000_0011u8,
    	]); // expect 304
    	assert!(unsigned(&mut buff).unwrap() == 500);
    }

    #[test]
    fn signed_reads_1() {
    	let mut buff = x(vec![1]);
    	assert!(signed(&mut buff).unwrap() == 1);
    }

    #[test]
    fn signed_not_reads_2() {
    	let mut buff = x(vec![1]);
    	assert!(signed(&mut buff).unwrap() != 2);	
    }
}
