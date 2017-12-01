extern crate leb;
use self::leb::signed;
use self::leb::unsigned;

use super::super::tree::language_types::{ValueType, LanguageType, ExternalKind};
use super::ParseError;

use tree::language_types::ResizableLimits;
use tree::language_types::GlobalType;
use parser::byteorder::ReadBytesExt;

use std::io::Read;

use parser::Parser;
impl<T: ValueType> Parser for T {
	type Item = Box<ValueType>;
	fn parse(reader: &mut Read) {

	}
}


impl ValueType {
	pub fn parse(reader: &mut Read) -> Result<Box<ValueType>, ParseError> {
		let read = signed(&mut reader.bytes())?;
		ValueType::get(read)
	}

	fn get(key: i64) -> Result<ValueType, ParseError> {
		Err(ParseError::WrongMagicNumber)
		// match key {
		// 	0x7f => Ok(ValueType::i_32),
		// 	0x7e => Ok(ValueType::i_64),
		// 	0x7d => Ok(ValueType::f_32),
		// 	0x7c => Ok(ValueType::f_64),
		// 	_    => Err(ParseError::InvalidValueType(key))
		// }
	}
}

impl LanguageType {
	pub fn parse(reader: &mut Read) -> Result<LanguageType, ParseError> {
		let read = signed(&mut reader.bytes())?;
		LanguageType::get(read)
	}

	fn get(key: i64) -> Result<LanguageType, ParseError> {
		Err(ParseError::WrongMagicNumber)
		// match key {
		// 	0x7f => Ok(LanguageType::i_32),
		// 	0x7e => Ok(LanguageType::i_64),
		// 	0x7d => Ok(LanguageType::f_32),
		// 	0x7c => Ok(LanguageType::f_64),
		// 	0x70 => Ok(LanguageType::anyfunc),
		// 	0x60 => Ok(LanguageType::func),
		// 	0x40 => Ok(LanguageType::empty_block),
		// 	_    => Err(ParseError::InvalidLanguageType(key))
		// }
	}
}

impl ExternalKind {
	pub fn parse(reader: &mut Read) -> Result<ExternalKind, ParseError> {
		Err(ParseError::WrongMagicNumber)
		// let external_kind = reader.read_u8()?;
		// let index = unsigned(&mut reader.bytes())?;
		// Ok(match external_kind {
		// 	0 => ExternalKind::function(index),
		// 	1 => ExternalKind::table(index),
		// 	2 => ExternalKind::memory(index),
		// 	3 => ExternalKind::global(index),
		// 	_ => return Err(ParseError::InvalidExternalKind(external_kind))
		// })
	}
}

impl ResizableLimits {
	pub fn parse(reader: &mut Read) -> Result<ResizableLimits, ParseError> {
		let flags = unsigned(&mut reader.bytes())?;
		let initial = unsigned(&mut reader.bytes())?;
        let maximum = if flags == 1 {
            Some(unsigned(&mut reader.bytes())?)
        } else {
            None
        };
        Ok(ResizableLimits{initial: initial, maximum:maximum})
	}
}

impl GlobalType {
	pub fn parse(reader: &mut Read) -> Result<GlobalType, ParseError> {
		let value_type = ValueType::parse(reader)?;
		let mutable = unsigned(&mut reader.bytes())?;
		Ok(GlobalType {
			contentType: value_type,
			mutability: mutable == 1
		})
	}
}