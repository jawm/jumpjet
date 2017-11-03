extern crate leb;
use self::leb::signed;

use super::super::tree::language_types::{ValueType, LanguageType};
use super::ParseError;

use std::io::Read;

impl ValueType {
	pub fn parse(reader: &mut Read) -> Result<ValueType, ParseError> {
		let read = signed(&mut reader.bytes())?;
		ValueType::get(read)
	}

	fn get(key: i64) -> Result<ValueType, ParseError> {
		match key {
			0x7f => Ok(ValueType::i_32),
			0x7e => Ok(ValueType::i_64),
			0x7d => Ok(ValueType::f_32),
			0x7c => Ok(ValueType::f_64),
			_    => Err(ParseError::InvalidValueType(key))
		}
	}
}

impl LanguageType {
	pub fn parse(reader: &mut Read) -> Result<LanguageType, ParseError> {
		let read = signed(&mut reader.bytes())?;
		LanguageType::get(read)
	}

	fn get(key: i64) -> Result<LanguageType, ParseError> {
		match key {
			0x7f => Ok(LanguageType::i_32),
			0x7e => Ok(LanguageType::i_64),
			0x7d => Ok(LanguageType::f_32),
			0x7c => Ok(LanguageType::f_64),
			0x70 => Ok(LanguageType::anyfunc),
			0x60 => Ok(LanguageType::func),
			0x40 => Ok(LanguageType::empty_block),
			_    => Err(ParseError::InvalidLanguageType(key))
		}
	}
}