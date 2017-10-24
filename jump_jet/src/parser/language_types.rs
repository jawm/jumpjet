extern crate leb;
use self::leb::signed;

use super::super::tree::language_types::*;
use super::ParseError;

use std::io::Read;


pub fn parse_value_type<T: Read>(reader: &mut T) -> Result<ValueType, ParseError> {
	let read = signed(&mut reader.bytes())?;
	get_value_type(read)
}

pub fn parse_type<T: Read>(reader: &mut T) -> Result<Type, ParseError> {
	let read = signed(&mut reader.bytes())?;
	get_type(read)
}

fn get_value_type(key: i64) -> Result<ValueType, ParseError> {
	match key {
		0x01 => Ok(ValueType::i_32),
		0x02 => Ok(ValueType::i_64),
		0x03 => Ok(ValueType::f_32),
		0x04 => Ok(ValueType::f_64),
		_    => Err(ParseError::InvalidValueType)
	}
}

fn get_type(key: i64) -> Result<Type, ParseError> {
	match key {
		0x01 => Ok(Type::i_32),
		0x02 => Ok(Type::i_64),
		0x03 => Ok(Type::f_32),
		0x04 => Ok(Type::f_64),
		0x10 => Ok(Type::anyfunc),
		0x20 => Ok(Type::func),
		0x40 => Ok(Type::empty_block),
		_    => Err(ParseError::InvalidLangaugeType)
	}
}