extern crate leb;
use self::leb::ReadLEB;

use super::super::tree::language_types::{ValueType, LanguageType, ExternalKind};
use super::ParseError;

use tree::language_types::ResizableLimits;
use tree::language_types::GlobalType;
use parser::byteorder::ReadBytesExt;

use std::io::Read;

impl ValueType {
	pub fn parse(reader: &mut Read) -> Result<ValueType, ParseError> {
		let read = reader.bytes().read_varint(7).unwrap();
		ValueType::get(read)
	}

	fn get(key: i64) -> Result<ValueType, ParseError> {
		match key {
			-0x01 => Ok(ValueType::i_32),
			-0x02 => Ok(ValueType::i_64),
			-0x03 => Ok(ValueType::f_32),
			-0x04 => Ok(ValueType::f_64),
			_    => Err(ParseError::InvalidValueType(key))
		}
	}
}

impl LanguageType {
	pub fn parse(reader: &mut Read) -> Result<LanguageType, ParseError> {
		let read = reader.bytes().read_varint(7).unwrap();
		LanguageType::get(read)
	}

	fn get(key: i64) -> Result<LanguageType, ParseError> {
		match key {
			-0x01 => Ok(LanguageType::value(ValueType::i_32)),
			-0x02 => Ok(LanguageType::value(ValueType::i_64)),
			-0x03 => Ok(LanguageType::value(ValueType::f_32)),
			-0x04 => Ok(LanguageType::value(ValueType::f_64)),
			-0x10 => Ok(LanguageType::anyfunc),
			-0x20 => Ok(LanguageType::func),
			-0x40 => Ok(LanguageType::empty_block),
			_    => Err(ParseError::InvalidLanguageType(key))
		}
	}
}

impl ExternalKind {
	pub fn parse(reader: &mut Read) -> Result<ExternalKind, ParseError> {
		let external_kind = reader.read_u8()?;

		// TODO this is only valid for external_kind of function.
		let index = reader.bytes().read_varuint(32).unwrap();
		Ok(match external_kind {
			0 => ExternalKind::function(index),
			1 => ExternalKind::table(index),
			2 => ExternalKind::memory(index),
			3 => ExternalKind::global(index),
			_ => return Err(ParseError::InvalidExternalKind(external_kind))
		})
	}
}

impl ResizableLimits {
	pub fn parse(reader: &mut Read) -> Result<ResizableLimits, ParseError> {
		let flags = reader.bytes().read_varuint(1).unwrap();
		let initial = reader.bytes().read_varuint(32).unwrap();
        let maximum = if flags == 1 {
            Some(reader.bytes().read_varuint(32).unwrap())
        } else {
            None
        };
        Ok(ResizableLimits{initial: initial, maximum:maximum})
	}
}

impl GlobalType {
	pub fn parse(reader: &mut Read) -> Result<GlobalType, ParseError> {
		let value_type = ValueType::parse(reader)?;
		let mutable = reader.bytes().read_varuint(1).unwrap();
		Ok(GlobalType {
			contentType: value_type,
			mutability: mutable == 1
		})
	}
}