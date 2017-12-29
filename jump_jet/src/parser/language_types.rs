use std::io::Bytes;
use std::io::Read;

use parser::byteorder::ReadBytesExt;
use parser::leb::ReadLEB;
use parser::ParseError;

use tree::language_types::{ValueType, LanguageType, ExternalKind, TableType};
use tree::language_types::ResizableLimits;
use tree::language_types::GlobalType;

impl ValueType {
	pub fn parse<R: Read>(bytes: &mut Bytes<R>) -> Result<ValueType, ParseError> {
		let read = bytes.read_varint(7).unwrap();
		ValueType::get(read)
	}

	fn get(key: i64) -> Result<ValueType, ParseError> {
		match key {
			-0x01 => Ok(ValueType::I32),
			-0x02 => Ok(ValueType::I64),
			-0x03 => Ok(ValueType::F32),
			-0x04 => Ok(ValueType::F64),
			_    => Err(ParseError::InvalidValueType(key))
		}
	}
}

impl LanguageType {
	pub fn parse<R: Read>(bytes: &mut Bytes<R>) -> Result<LanguageType, ParseError> {
		let read = bytes.read_varint(7).unwrap();
		LanguageType::get(read)
	}

	fn get(key: i64) -> Result<LanguageType, ParseError> {
		match key {
			-0x01 => Ok(LanguageType::Value(ValueType::I32)),
			-0x02 => Ok(LanguageType::Value(ValueType::I64)),
			-0x03 => Ok(LanguageType::Value(ValueType::F32)),
			-0x04 => Ok(LanguageType::Value(ValueType::F64)),
			-0x10 => Ok(LanguageType::Anyfunc),
			-0x20 => Ok(LanguageType::Func),
			-0x40 => Ok(LanguageType::EmptyBlock),
			_    => Err(ParseError::InvalidLanguageType(key))
		}
	}
}

impl ExternalKind {
	pub fn parse(reader: &mut Read) -> Result<ExternalKind, ParseError> {
		let external_kind = reader.read_u8()?;
		Ok(match external_kind {
			0 => ExternalKind::Function(reader.bytes().read_varuint(32)? as usize),
			1 => ExternalKind::Table(reader.bytes().read_varuint(32)? as usize),
			2 => ExternalKind::Memory(reader.bytes().read_varuint(32)? as usize),
			3 => ExternalKind::Global(reader.bytes().read_varuint(32)? as usize),
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

// impl MemoryType {
// 	pub fn parse(reader: &mut Read) -> Result<MemoryType, ParseError> {
// 		let limits_res = ResizableLimits::parse(reader);
// 		match limits_res {
// 			Ok(limits) => Ok(MemoryType{limits}),
// 			Err(e) => Err(e)
// 		}
// 	}
// }

impl GlobalType {
	pub fn parse(reader: &mut Read) -> Result<GlobalType, ParseError> {
		let value_type = ValueType::parse(&mut reader.bytes())?;
		let mutable = reader.bytes().read_varuint(1).unwrap();
		Ok(GlobalType {
			content_type: value_type,
			mutability: mutable == 1
		})
	}
}

impl TableType {
	pub fn parse(reader: &mut Read) -> Result<TableType, ParseError> {
		let elem_type_res = LanguageType::parse(&mut reader.bytes());
		match elem_type_res {
			Ok(elem_type) => {
				if elem_type != LanguageType::Anyfunc {
					return Err(ParseError::CustomError("WASM 1.0 only valid table type is `anyfunc`".to_string()));
				}
				let limits_res = ResizableLimits::parse(reader);
				match limits_res {
					Ok(limits) => Ok(TableType{elem_type, limits}),
					Err(e) => Err(e)
				}
			},
			Err(e) => Err(e)
		}
	}
}