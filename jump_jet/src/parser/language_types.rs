extern crate leb;
use self::leb::signed;

use super::super::tree::language_types::{ValueType, LanguageType};
use super::ParseError;

use std::io::Read;

impl ValueType {
	pub fn parse<T: Read>(reader: &mut T) -> Result<ValueType, ParseError> {
		let read = signed(&mut reader.bytes())?;
		ValueType::get(read)
	}

	fn get(key: i64) -> Result<ValueType, ParseError> {
		match key {
			0x01 => Ok(ValueType::i_32),
			0x02 => Ok(ValueType::i_64),
			0x03 => Ok(ValueType::f_32),
			0x04 => Ok(ValueType::f_64),
			_    => Err(ParseError::InvalidValueType)
		}
	}
}

impl LanguageType {
	pub fn parse<T: Read>(reader: &mut T) -> Result<LanguageType, ParseError> {
		let read = signed(&mut reader.bytes())?;
		LanguageType::get(read)
	}

	fn get(key: i64) -> Result<LanguageType, ParseError> {
		match key {
			0x01 => Ok(LanguageType::i_32),
			0x02 => Ok(LanguageType::i_64),
			0x03 => Ok(LanguageType::f_32),
			0x04 => Ok(LanguageType::f_64),
			0x10 => Ok(LanguageType::anyfunc),
			0x20 => Ok(LanguageType::func),
			0x40 => Ok(LanguageType::empty_block),
			_    => Err(ParseError::InvalidLangaugeType)
		}
	}
}