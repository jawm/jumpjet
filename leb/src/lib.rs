use std::io::{Bytes, Cursor, Error, Read, ErrorKind};

const CONTINUE_MASK: u64 = 0x80;
const VALUE_MASK: u64 = 0x7F;

pub trait ReadLEB : Iterator {
    fn read_varuint<R: Read>(&mut self, mut max_bits: i8, buffer: &mut Bytes<R>) -> Result<u64, Error> {
        let mut result: u64 = 0;
        let mut shift = 0;
        while max_bits > 0 {
            let next = buffer.next().unwrap().unwrap() as u64;
            result |= (next & VALUE_MASK) << shift;
            if next & CONTINUE_MASK == 0 {
                if max_bits < 8 {
                    if next & (0xff << max_bits) != 0 {
                        return Err(Error::new(ErrorKind::Other,"Wrong value pal"));
                    }
                }
                return Ok(result)
            }
            shift += 7;
            max_bits -= 7;
        }
        println!("num too big {:?}, {:?}", max_bits, shift);
        Err(Error::new(ErrorKind::Other, "Num too big"))
    }

    fn read_varint<R: Read>(&mut self, max_bits: i8, buffer: &mut Bytes<R>) -> Result<i64, Error> {
        match self.read_varuint(max_bits, buffer) {
            Ok(val) => Ok(val as i64),
            Err(e) => Err(e)
        }
    }
}

pub fn read<R: Read>(mut max_bits: i8, buffer: &mut Bytes<R>) -> Result<u64, Error> {
    let mut result: u64 = 0;
    let mut shift = 0;
    while max_bits > 0 {
        let next = buffer.next().unwrap().unwrap() as u64;
        result |= (next & VALUE_MASK) << shift;
        if next & CONTINUE_MASK == 0 {
            if max_bits < 8 {
                if next & (0xff << max_bits) != 0 {
                    return Err(Error::new(ErrorKind::Other,"Wrong value pal"));
                }
            }
            return Ok(result)
        }
        shift += 7;
        max_bits -= 7;
    }
    println!("num too big {:?}, {:?}", max_bits, shift);
    Err(Error::new(ErrorKind::Other, "Num too big"))
}

impl<R: Read> ReadLEB for Bytes<R> {
    fn read_varuint<S: Read>(&mut self, mut max_bits: i8, buffer: &mut Bytes<S>) -> Result<u64, Error> {
        let mut result: u64 = 0;
        let mut shift = 0;
        while max_bits > 0 {
            let next = self.next().unwrap().unwrap() as u64;
            result |= (next & VALUE_MASK) << shift;
            if next & CONTINUE_MASK == 0 {
                if max_bits < 8 {
                    if next & (0xff << max_bits) != 0 {
                        return Err(Error::new(ErrorKind::Other,"Wrong value pal"));
                    }
                }
                return Ok(result)
            }
            shift += 7;
            max_bits -= 7;
        }
        println!("num too big {:?}, {:?}", max_bits, shift);
        Err(Error::new(ErrorKind::Other, "Num too big"))
    }

    fn read_varint<S: Read>(&mut self, max_bits: i8, buffer: &mut Bytes<S>) -> Result<i64, Error> {
        match self.read_varuint(max_bits, buffer) {
            Ok(val) => Ok(val as i64),
            Err(e) => Err(e)
        }
    }
}

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
    use std::io::{Bytes, Cursor, Read};

    fn b(bytes: &[u8]) -> Bytes<Cursor<Vec<u8>>> {
        Cursor::new(bytes.to_vec()).bytes()
    }

    #[test]
    fn test_unsigned_decode() {
        assert!(read(1, &mut b(&[0])).unwrap() == 0);
        assert!(b(&[0]).read_varuint(1).unwrap() == 0);

        assert!(read(1, &mut b(&[1])).unwrap() == 1);
        assert!(read(7, &mut b(&[42])).unwrap() == 42);
        assert!(read(7, &mut b(&[127])).unwrap() == 127);
        assert!(read(32, &mut b(&[128, 1])).unwrap() == 128);
        assert!(read(32, &mut b(&[255, 1])).unwrap() == 255);

        assert!(read(32, &mut b(&[0])).unwrap() == 0);
        assert!(read(32, &mut b(&[42])).unwrap() == 42);
        assert!(read(32, &mut b(&[127])).unwrap() == 127);
        assert!(read(32, &mut b(&[128, 1])).unwrap() == 128);
        assert!(read(32, &mut b(&[255, 255, 3])).unwrap() == 0xffff);
        assert!(read(32, &mut b(&[0xE5, 0x8E, 0x26])).unwrap() == 624485);
        assert!(read(32, &mut b(&[255, 255, 255, 255, 0b1111])).unwrap() == 0xffff_ffff);
        assert!(read(64, &mut b(&[255, 255, 255, 255, 255, 255, 255, 255, 255, 1])).unwrap() == 0xffff_ffff_ffff_ffff);
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_u1() {
        read(1, &mut b(&[2])).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_u7() {
        read(7, &mut b(&[128])).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_u8() {
        read(8, &mut b(&[128, 2])).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_u16() {
        read(16, &mut b(&[128, 128, 4])).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_u32() {
        read(32, &mut b(&[128, 128, 128, 128, 16])).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_u64() {
        read(16, &mut b(&[128, 128, 128, 128, 128, 128, 128, 128, 128, 2])).unwrap();
    }
}
