use std::io::Bytes;
use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

pub fn read_string(reader: &mut Read) -> Result<String,ParseError> {
    let field_len = reader.bytes().read_varuint(32).unwrap();
    let mut field = "".to_string();
    match reader.take(field_len).read_to_string(&mut field) {
    	Ok(_) => Ok(field),
    	Err(e) => Err(ParseError::Io(e))
    }
}

pub fn swallow_expr<R: Read>(bytes: &mut Bytes<R>) -> i64 {
    while bytes.next().unwrap().unwrap() != 0x0b {};
    0
}