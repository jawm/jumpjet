use std::io::{
    Bytes,
    Error,
    Read,
    ErrorKind
};

const CONTINUE_MASK: u64 = 0x80;
const VALUE_MASK: u64 = 0x7F;
const SIGN_MASK: u64 = 0x40;

fn read_values<R: Read>(buffer: &mut Bytes<R>, mut max_bits: i8) -> Result<(u64,u64,u64),Error> {
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
            return Ok((result,next,shift));
        }
        shift += 7;
        max_bits -= 7;
    }
    Err(Error::new(ErrorKind::Other, "Num too big"))
}

pub trait ReadLEB : Iterator {
    fn read_varuint(&mut self, max_bits: i8) -> Result<u64, Error>;

    fn read_varint(&mut self, max_bits: i8) -> Result<i64, Error>;
}

impl<R: Read> ReadLEB for Bytes<R> {
    fn read_varuint(&mut self, max_bits: i8) -> Result<u64, Error> {
        match read_values(self, max_bits) {
            Ok((result,_,_)) => Ok(result),
            Err(e) => Err(e)
        }
    }

    fn read_varint(&mut self, max_bits: i8) -> Result<i64, Error> {
        match read_values(self, max_bits) {
            Ok((mut result,next,shift)) => {
                if next & SIGN_MASK != 0 {
                    result |= !0 << (shift+7);
                }
                Ok(result as i64)
            },
            Err(e) => Err(e)
        }
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
        assert!(b(&[0]).read_varuint(1).unwrap() == 0);

        assert!(b(&[1]).read_varuint(1).unwrap() == 1);
        assert!(b(&[42]).read_varuint(7).unwrap() == 42);
        assert!(b(&[127]).read_varuint(7).unwrap() == 127);
        assert!(b(&[128, 1]).read_varuint(32).unwrap() == 128);
        assert!(b(&[255, 1]).read_varuint(32).unwrap() == 255);

        assert!(b(&[0]).read_varuint(32).unwrap() == 0);
        assert!(b(&[42]).read_varuint(32).unwrap() == 42);
        assert!(b(&[127]).read_varuint(32).unwrap() == 127);
        assert!(b(&[128, 1]).read_varuint(32).unwrap() == 128);
        assert!(b(&[255, 255, 3]).read_varuint(32).unwrap() == 0xffff);
        assert!(b(&[0xE5, 0x8E, 0x26]).read_varuint(32).unwrap() == 624485);
        assert!(b(&[255, 255, 255, 255, 0b1111]).read_varuint(32).unwrap() == 0xffff_ffff);
        assert!(b(&[255, 255, 255, 255, 255, 255, 255, 255, 255, 1]).read_varuint(64).unwrap() == 0xffff_ffff_ffff_ffff);
    }

    #[test]
    fn test_signed_decode() {
        assert!(b(&[0]).read_varint(1).unwrap() == 0);
        assert!(b(&[1]).read_varint(1).unwrap() == 1);
        assert!(b(&[2]).read_varint(7).unwrap() == 2);
        assert!(b(&[0x7e]).read_varint(7).unwrap() == -2);
        assert!(b(&[0xff, 0]).read_varint(16).unwrap() == 127);
        assert!(b(&[0x81, 0x7f]).read_varint(16).unwrap() == -127);
        assert!(b(&[0x80, 0x7f]).read_varint(16).unwrap() == -128);
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_u1() {
        b(&[2]).read_varuint(1).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_u7() {
        b(&[128]).read_varuint(7).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_u8() {
        b(&[128, 2]).read_varuint(8).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_u16() {
        b(&[128, 128, 4]).read_varuint(16).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_u32() {
        b(&[128, 128, 128, 128, 16]).read_varuint(32).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_u64() {
        b(&[128, 128, 128, 128, 128, 128, 128, 128, 128, 2]).read_varuint(64).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_i8() {
        b(&[128, 2]).read_varint(8).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_i16() {
        b(&[128, 128, 4]).read_varint(16).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_i32() {
        b(&[128, 128, 128, 128, 16]).read_varint(32).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_decode_overflow_i64() {
        b(&[128, 128, 128, 128, 128, 128, 128, 128, 128, 2]).read_varint(64).unwrap();
    }
}
