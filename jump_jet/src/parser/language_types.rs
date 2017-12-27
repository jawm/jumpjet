use std::io::Bytes;
use std::io::Read;

use parser::byteorder::ReadBytesExt;
use parser::leb::ReadLEB;
use parser::ParseError;

use tree::functions::FunctionSection;
use tree::globals::GlobalSection;
use tree::language_types::{ValueType, LanguageType, ExternalKind, MemoryType, TableType};
use tree::language_types::ResizableLimits;
use tree::language_types::GlobalType;
use tree::memory::MemorySection;
use tree::Module;
use tree::tables::TableSection;

impl ValueType {
	pub fn parse<R: Read>(bytes: &mut Bytes<R>) -> Result<ValueType, ParseError> {
		let read = bytes.read_varint(7).unwrap();
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
	pub fn parse<R: Read>(bytes: &mut Bytes<R>) -> Result<LanguageType, ParseError> {
		let read = bytes.read_varint(7).unwrap();
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
	pub fn parse(reader: &mut Read, module: &Module) -> Result<ExternalKind, ParseError> {
		let external_kind = reader.read_u8()?;

		// TODO this is only valid for external_kind of function.
		// Ok(match external_kind {
		// 	0 => ExternalKind::function(
		// 		module.get_section::<FunctionSection>(3).unwrap().functions[
		// 			reader.bytes().read_varuint(32)? as usize
		// 		]
		// 	),
		// 	1 => ExternalKind::table(TableType::parse(reader)?),
		// 	2 => ExternalKind::memory(MemoryType::parse(reader)?),
		// 	3 => ExternalKind::global(GlobalType::parse(reader)?),
		// 	_ => return Err(ParseError::InvalidExternalKind(external_kind))
		// })
		Err(ParseError::CustomError("commented out line 97".to_string()))
	}

	pub fn by_index(reader: &mut Read, module: &Module) -> Result<ExternalKind, ParseError> {
		let external_kind = reader.read_u8()?;
		// Ok(match external_kind {
		// 	0 => ExternalKind::function(
		// 		module.get_section::<FunctionSection>(3).unwrap().functions[
		// 			reader.bytes().read_varuint(32)? as usize
		// 		]
		// 	),
		// 	1 => ExternalKind::table(
		// 		module.get_section::<TableSection>(3).unwrap().entries[
		// 			reader.bytes().read_varuint(32)? as usize
		// 		]
		// 	),
		// 	2 => ExternalKind::memory(
		// 		module.get_section::<MemorySection>(3).unwrap().entries[
		// 			reader.bytes().read_varuint(32)? as usize
		// 		]
		// 	),
		// 	3 => ExternalKind::global(
		// 		module.get_section::<GlobalSection>(3).unwrap().entries[
		// 			reader.bytes().read_varuint(32)? as usize
		// 		].data_type
		// 	),
		// 	_ => return Err(ParseError::InvalidExternalKind(external_kind))
		// })
		Err(ParseError::CustomError("commented out line 97".to_string()))
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

impl MemoryType {
	pub fn parse(reader: &mut Read) -> Result<MemoryType, ParseError> {
		let limits_res = ResizableLimits::parse(reader);
		match limits_res {
			Ok(limits) => Ok(MemoryType{limits}),
			Err(e) => Err(e)
		}
	}
}

impl GlobalType {
	pub fn parse(reader: &mut Read) -> Result<GlobalType, ParseError> {
		let value_type = ValueType::parse(&mut reader.bytes())?;
		let mutable = reader.bytes().read_varuint(1).unwrap();
		Ok(GlobalType {
			contentType: value_type,
			mutability: mutable == 1
		})
	}
}

impl TableType {
	pub fn parse(reader: &mut Read) -> Result<TableType, ParseError> {
		let elem_type_res = LanguageType::parse(&mut reader.bytes());
		match elem_type_res {
			Ok(elem_type) => {
				if elem_type != LanguageType::anyfunc {
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