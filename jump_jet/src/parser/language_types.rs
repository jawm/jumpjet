use std::io::Bytes;
use std::io::Read;

use parser::byteorder::LittleEndian;
use parser::byteorder::ReadBytesExt;
use parser::leb::ReadLEB;
use parser::ParseError;

use tree::language_types::BlockType;
use tree::language_types::BranchTable;
use tree::language_types::ExternalKind;
use tree::language_types::GlobalType;
use tree::language_types::InitExpression;
use tree::language_types::LanguageType;
use tree::language_types::Operation;
use tree::language_types::ResizableLimits;
use tree::language_types::TableType;
use tree::language_types::ValueType;
use tree::Module;

impl ValueType {
	pub fn parse<R: Read>(bytes: &mut Bytes<R>) -> Result<ValueType, ParseError> {
		let read = bytes.read_varint(7).unwrap();
		ValueType::get(read)
	}

	pub fn get(key: i64) -> Result<ValueType, ParseError> {
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

impl InitExpression {
	pub fn parse(reader: &mut Read, module: &Module) -> Result<InitExpression, ParseError> {
		let byte = reader.bytes().next().unwrap().unwrap();
		match byte {
			0x41 => {
				let immediate = reader.bytes().read_varint(32).unwrap() as i32;
				let end_op = reader.bytes().next().unwrap().unwrap();
				if end_op != 0x0b {
					Err(ParseError::CustomError("invalid i32.const instruction in init expression".to_string()))
				} else {
					Ok(InitExpression::I32Const(immediate))
				}
			},
			0x42 => {
				let immediate = reader.bytes().read_varint(64).unwrap();
				let end_op = reader.bytes().next().unwrap().unwrap();
				if end_op != 0x0b {
					Err(ParseError::CustomError("invalid i64.const instruction in init expression".to_string()))
				} else {
					Ok(InitExpression::I64Const(immediate))
				}
			},
			0x43 => {
				let immediate = reader.read_u32::<LittleEndian>().unwrap() as f32;
				let end_op = reader.bytes().next().unwrap().unwrap();
				if end_op != 0x0b {
					Err(ParseError::CustomError("invalid f32.const instruction in init expression".to_string()))
				} else {
					Ok(InitExpression::F32Const(immediate))
				}
			},
			0x44 => {
				let immediate = reader.read_u64::<LittleEndian>().unwrap() as f64;
				let end_op = reader.bytes().next().unwrap().unwrap();
				if end_op != 0x0b {
					Err(ParseError::CustomError("invalid f64.const instruction in init expression".to_string()))
				} else {
					Ok(InitExpression::F64Const(immediate))
				}
			},
			0x23 => {
				let immediate = reader.bytes().read_varint(32).unwrap() as usize;
				let end_op = reader.bytes().next().unwrap().unwrap();
				if end_op != 0x0b {
					Err(ParseError::CustomError("invalid get_global instruction in init expression".to_string()))
				} else if module.globals[immediate].constraints.mutability {
					Err(ParseError::CustomError("get_global in init expressions can only refer to immutable globals".to_string()))
				} else {
					Ok(InitExpression::GetGlobal(immediate))
				}
			},
			_ => Err(ParseError::CustomError("Unexpected byte in init expression".to_string()))
		}
	}
}

impl Operation {
	pub fn parse_multiple(reader: &mut Read, module: &Module) -> Result<Vec<Operation>, ParseError> {
		let mut ops = vec![];
		loop {
			match Operation::parse(reader, module) {
				Ok(operation) => {
					if let Operation::End = operation {
						ops.push(operation);
						break;
					}
					ops.push(operation);
				},
				Err(e) => {return Err(e);}
			}
		}
		println!("/break");
		Ok(ops)
	}

	pub fn parse(reader: &mut Read, module: &Module) -> Result<Operation, ParseError> {
		let opcode = reader.bytes().next().unwrap().unwrap();
		print!("{:X} ", opcode);
		match opcode {
			0x00 => Ok(Operation::Unreachable),
			0x01 => Ok(Operation::Nop),
			0x02 => match BlockType::parse(reader, module) {
				Ok(block) => Ok(Operation::Block(block)),
				Err(e) => Err(e)
			},
			0x03 => match BlockType::parse(reader, module) {
				Ok(block) => Ok(Operation::Loop(block)),
				Err(e) => Err(e)
			},
			0x04 => match BlockType::parse(reader, module) {
				Ok(block) => Ok(Operation::If(block)),
				Err(e) => Err(e)
			},
			0x05 => Ok(Operation::Else),
			0x0b => Ok(Operation::End),
			0x0c => Ok(Operation::Branch(reader.bytes().read_varuint(32).unwrap() as u32)),
			0x0d => Ok(Operation::BranchIf(reader.bytes().read_varuint(32).unwrap() as u32)),
			0x0e => {
				match BranchTable::parse(reader, module) {
					Ok(branch_table) => Ok(Operation::BranchTable(branch_table)),
					Err(e) => Err(e)
				}
			},
			0x0f => Ok(Operation::Return),
			0x11 => {
				let type_index = reader.bytes().read_varuint(32).unwrap() as usize;
				let reserved = reader.bytes().read_varuint(1).unwrap() == 1;
				if reserved {
					return Err(ParseError::CustomError("call_indirect reserved field must be 0".to_string()));
				}
				Ok(Operation::CallIndirect(type_index, reserved))

			}
			0x20 => {
				let immediate = reader.bytes().read_varuint(32).unwrap() as u32;
				Ok(Operation::GetLocal(immediate))
			}
			0x41 => {
				let immediate = reader.bytes().read_varint(32).unwrap() as i32;
				Ok(Operation::I32Const(immediate))
			},
			0x42 => {
				let immediate = reader.bytes().read_varint(64).unwrap();
				Ok(Operation::I64Const(immediate))
			},
			0x43 => {
				let immediate = reader.read_u32::<LittleEndian>().unwrap() as f32;
				Ok(Operation::F32Const(immediate))
			},
			0x44 => {
				let immediate = reader.read_u64::<LittleEndian>().unwrap() as f64;
				Ok(Operation::F64Const(immediate))
			},
			_ => Err(ParseError::CustomError("Unknown opcode".to_string()))
		}
	}
}

impl BlockType {
	pub fn parse(reader: &mut Read, module: &Module) -> Result<BlockType, ParseError> {
		let byte = reader.bytes().read_varint(7).unwrap();
		if let Ok(value_type) = ValueType::get(byte) {
			Ok(BlockType::Value(value_type))
		} else if byte as u8 == 0x40 {
			Ok(BlockType::Empty)
		} else {
			Err(ParseError::CustomError("Block type wasn't valid".to_string()))
		}
	}
}

impl BranchTable {
	pub fn parse(reader: &mut Read, module: &Module) -> Result<BranchTable, ParseError> {
		let target_count = reader.bytes().read_varuint(32).unwrap() as u32;
		let mut targets = vec![];
		for _ in 0..target_count {
			targets.push(reader.bytes().read_varuint(32).unwrap() as u32);
		}
		let default = reader.bytes().read_varuint(32).unwrap() as usize;
		Ok(BranchTable {
			targets,
			default
		})
	}
}