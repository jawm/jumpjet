use std::io::{Bytes, Error, Read, ErrorKind};

const CONTINUE_MASK: u64 = 0x80;
const VALUE_MASK: u64 = 0x7F;

/*
A LEB128 variable-length integer, limited to N bits (i.e., the values [0, 2^N-1]), represented by at most ceil(N/7) bytes that may contain padding 0x80 bytes.

A max of N bits for the int?


*/

#[derive(Debug)]
struct vs7(i8);

#[derive(Debug)]
struct vs32(i32);

#[derive(Debug)]
struct vs64(i64);

#[derive(Debug)]
struct vu1(u8);
impl vu1 {
    pub fn parse() -> Result<vu1> {
        
    }
}

#[derive(Debug)]
struct vu7(u8);

#[derive(Debug)]
struct vu32(u32);

#[derive(Debug)]
struct vu64(u64);

pub fn signed<R: Read>(buffer: &mut Bytes<R>) -> Result<i64, Error> {
	match unsigned(buffer) {
		Ok(val) => Ok(val as i64),
		Err(e) => Err(e)
	}
}

pub fn unsigned<R: Read>(buffer: &mut Bytes<R>) -> Result<u64, Error> {
	let mut result: u64 = 0;
	let mut shift = 0;
    loop {
        let byte = match buffer.next() {
            Some(t) => match t {
                Ok(v) => v as u64,
                Err(e) => return Err(e)
            },
            None => return Err(Error::new(ErrorKind::Other, "Failed reading unsigned"))
        };
        result |= (byte & VALUE_MASK) << shift;
        if byte & CONTINUE_MASK == 0 {
            break;
        }
        shift += 7;
    }
	Ok(result)
}

#[derive(Debug)]
pub struct VarUInt(u32);

impl VarUInt {
    pub fn parse(mut bits: i8, buffer: &mut Read) -> Result<VarUInt, Error> {
        assert!(bits > 0);
        let mut result: u64 = 0;
        let mut shift = 0;
        let mut bytes = buffer.bytes();
        loop {
            let byte = match bytes.next() {
                Some(t) => match t {
                    Ok(v) => v as u64,
                    Err(e) => return Err(e)
                },
                None => return Err(Error::new(ErrorKind::Other, "Failed reading unsigned"))
            };
            result |= (byte & VALUE_MASK) << shift;
            //bits -= 8;
            if byte & CONTINUE_MASK == 0 {
                if bits < 8 {
                    let xmask = 0xff << bits;
                    if byte & xmask != 0 {
                        return Err(Error::new(ErrorKind::Other, "Consumed more bits than allowed"));
                    }
                }
                break;
            } else if bits < 0 { // it *should* have broken
                return Err(Error::new(ErrorKind::Other, "Continue flag present beyond max bits to read."))
            }
            bits -= 8;
            shift += 7;
        }
        Ok(VarUInt(result as u32))
    }
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::{Bytes, Cursor, Error, Read};

    fn test_varuint(max: i8, to_read: Vec<u8>) -> Result<VarUInt,Error> {
        let mut c = Cursor::new(to_read);
        VarUInt::parse(max, &mut c)
    }

    #[test]
    fn test_varuint_does_read() {
        assert!(test_varuint(1,  vec![0]).unwrap().0 == 0);
        assert!(test_varuint(1,  vec![1]).unwrap().0 == 1);
        assert!(test_varuint(7,  vec![1]).unwrap().0 == 1);
        assert!(test_varuint(7,  vec![7]).unwrap().0 == 7);
        assert!(test_varuint(7,  vec![127]).unwrap().0 == 127);
        assert!(test_varuint(31, vec![]))
    }

    #[test]
    fn test_varuint_doesnt_read() {
        assert!(test_varuint(1, vec![7]).is_err());
    }


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
