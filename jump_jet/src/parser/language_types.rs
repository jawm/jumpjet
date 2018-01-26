use std::io::Bytes;
use std::io::Read;

use parser::byteorder::LittleEndian;
use parser::byteorder::ReadBytesExt;
use parser::leb::ReadLEB;
use parser::ParseError;

use parse_tree::language_types::BlockType;
use parse_tree::language_types::BranchTable;
use parse_tree::language_types::ExternalKind;
use parse_tree::language_types::GlobalType;
use parse_tree::language_types::InitExpression;
use parse_tree::language_types::LanguageType;
use parse_tree::language_types::MemoryImmediate;
use parse_tree::language_types::Operation;
use parse_tree::language_types::ResizableLimits;
use parse_tree::language_types::TableType;
use parse_tree::language_types::ValueType;
use parse_tree::ParseModule;

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
		println!("attempting");
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
	pub fn parse(reader: &mut Read, module: &ParseModule) -> Result<InitExpression, ParseError> {
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
	pub fn parse_multiple(reader: &mut Read, module: &ParseModule) -> Result<Vec<Operation>, ParseError> {
		let mut ops = vec![];
		let mut ends_required = 1;
		loop {
			match Operation::parse(reader, module) {
				Ok(operation) => {
					match operation {
						Operation::End => ends_required -= 1,
						Operation::Block(_) | Operation::Loop(_) | Operation::If(_) => ends_required += 1,
						_ => {}
					}
					ops.push(operation);
					if ends_required == 0 {
						break;
					}
				},
				Err(e) => {return Err(e);}
			}
		}
		Ok(ops)
	}

	pub fn parse(reader: &mut Read, module: &ParseModule) -> Result<Operation, ParseError> {
		let opcode = reader.bytes().next().unwrap().unwrap();
		match opcode {

			// Control flow operators
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

			// Call operators
			0x10 => {
				let function_index = reader.bytes().read_varuint(32).unwrap() as usize;
				Ok(Operation::Call(function_index))
			}
			0x11 => {
				let type_index = reader.bytes().read_varuint(32).unwrap() as usize;
				let reserved = reader.bytes().read_varuint(1).unwrap() == 1;
				if reserved {
					return Err(ParseError::CustomError("call_indirect reserved field must be 0".to_string()));
				}
				Ok(Operation::CallIndirect(type_index, reserved))

			},

			// Parametric operators
			0x1a => Ok(Operation::Drop),
			0x1b => Ok(Operation::Select),

			// Variable access
			0x20 => {
				let immediate = reader.bytes().read_varuint(32).unwrap() as usize;
				Ok(Operation::GetLocal(immediate))
			},
			0x21 => {
				let immediate = reader.bytes().read_varuint(32).unwrap() as usize;
				Ok(Operation::SetLocal(immediate))
			},
			0x22 => {
				let immediate = reader.bytes().read_varuint(32).unwrap() as usize;
				Ok(Operation::TeeLocal(immediate))
			},
			0x23 => {
				let immediate = reader.bytes().read_varuint(32).unwrap() as usize;
				Ok(Operation::GetGlobal(immediate))
			},
			0x24 => {
				let immediate = reader.bytes().read_varuint(32).unwrap() as usize;
				Ok(Operation::SetGlobal(immediate))
			},

			// Memory-related operators
			0x28 => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I32Load(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x29 => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I64Load(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x2a => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::F32Load(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x2b => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::F64Load(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x2c => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I32Load8S(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x2d => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I32Load8U(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x2e => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I32Load16S(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x2f => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I32Load16U(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x30 => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I64Load8S(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x31 => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I64Load8U(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x32 => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I64Load16S(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x33 => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I64Load16U(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x34 => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I64Load32S(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x35 => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I64Load32U(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x36 => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I32Store(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x37 => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I64Store(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x38 => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::F32Store(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x39 => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::F64Store(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x3a => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I32Store8(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x3b => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I32Store16(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x3c => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I64Store8(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x3d => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I64Store16(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x3e => {
				match MemoryImmediate::parse(reader, module) {
					Ok(memory_immediate) => Ok(Operation::I64Store32(memory_immediate)),
					Err(e) => Err(e)
				}
			},
			0x3f => {
				let reserved = reader.bytes().read_varuint(1).unwrap() == 1;
				Ok(Operation::CurrentMemory(reserved))
			},
			0x40 => {
				let reserved = reader.bytes().read_varuint(1).unwrap() == 1;
				Ok(Operation::GrowMemory(reserved))
			},

			// Constants
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

			// Comparison operators
			0x45 => Ok(Operation::I32Eqz),
			0x46 => Ok(Operation::I32Eq),
			0x47 => Ok(Operation::I32Ne),
			0x48 => Ok(Operation::I32LtS),
			0x49 => Ok(Operation::I32LtU),
			0x4a => Ok(Operation::I32GtS),
			0x4b => Ok(Operation::I32GtU),
			0x4c => Ok(Operation::I32LeS),
			0x4d => Ok(Operation::I32LeU),
			0x4e => Ok(Operation::I32GeS),
			0x4f => Ok(Operation::I32GeU),
			0x50 => Ok(Operation::I64Eqz),
			0x51 => Ok(Operation::I64Eq),
			0x52 => Ok(Operation::I64Ne),
			0x53 => Ok(Operation::I64LtS),
			0x54 => Ok(Operation::I64LtU),
			0x55 => Ok(Operation::I64GtS),
			0x56 => Ok(Operation::I64GtU),
			0x57 => Ok(Operation::I64LeS),
			0x58 => Ok(Operation::I64LeU),
			0x59 => Ok(Operation::I64GeS),
			0x5a => Ok(Operation::I64GeU),
			0x5b => Ok(Operation::F32Eq),
			0x5c => Ok(Operation::F32Ne),
			0x5d => Ok(Operation::F32Lt),
			0x5e => Ok(Operation::F32Gt),
			0x5f => Ok(Operation::F32Le),
			0x60 => Ok(Operation::F32Ge),
			0x61 => Ok(Operation::F64Eq),
			0x62 => Ok(Operation::F64Ne),
			0x63 => Ok(Operation::F64Lt),
			0x64 => Ok(Operation::F64Gt),
			0x65 => Ok(Operation::F64Le),
			0x66 => Ok(Operation::F64Ge),

			// Numeric operators
			0x67 => Ok(Operation::I32Clz),
			0x68 => Ok(Operation::I32Ctz),
			0x69 => Ok(Operation::I32Popcnt),
			0x6a => Ok(Operation::I32Add),
			0x6b => Ok(Operation::I32Sub),
			0x6c => Ok(Operation::I32Mul),
			0x6d => Ok(Operation::I32DivS),
			0x6e => Ok(Operation::I32DivU),
			0x6f => Ok(Operation::I32RemS),
			0x70 => Ok(Operation::I32RemU),
			0x71 => Ok(Operation::I32And),
			0x72 => Ok(Operation::I32Or),
			0x73 => Ok(Operation::I32Xor),
			0x74 => Ok(Operation::I32Shl),
			0x75 => Ok(Operation::I32ShrS),
			0x76 => Ok(Operation::I32ShrU),
			0x77 => Ok(Operation::I32Rotl),
			0x78 => Ok(Operation::I32Rotr),
			0x79 => Ok(Operation::I64Clz),
			0x7a => Ok(Operation::I64Ctz),
			0x7b => Ok(Operation::I64Popcnt),
			0x7c => Ok(Operation::I64Add),
			0x7d => Ok(Operation::I64Sub),
			0x7e => Ok(Operation::I64Mul),
			0x7f => Ok(Operation::I64DivS),
			0x80 => Ok(Operation::I64DivU),
			0x81 => Ok(Operation::I64RemS),
			0x82 => Ok(Operation::I64RemU),
			0x83 => Ok(Operation::I64And),
			0x84 => Ok(Operation::I64Or),
			0x85 => Ok(Operation::I64Xor),
			0x86 => Ok(Operation::I64Shl),
			0x87 => Ok(Operation::I64ShrS),
			0x88 => Ok(Operation::I64ShrU),
			0x89 => Ok(Operation::I64Rotl),
			0x8a => Ok(Operation::I64Rotr),
			0x8b => Ok(Operation::F32Abs),
			0x8c => Ok(Operation::F32Neg),
			0x8d => Ok(Operation::F32Ceil),
			0x8e => Ok(Operation::F32Floor),
			0x8f => Ok(Operation::F32Trunc),
			0x90 => Ok(Operation::F32Nearest),
			0x91 => Ok(Operation::F32Sqrt),
			0x92 => Ok(Operation::F32Add),
			0x93 => Ok(Operation::F32Sub),
			0x94 => Ok(Operation::F32Mul),
			0x95 => Ok(Operation::F32Div),
			0x96 => Ok(Operation::F32Min),
			0x97 => Ok(Operation::F32Max),
			0x98 => Ok(Operation::F32Copysign),
			0x99 => Ok(Operation::F64Abs),
			0x9a => Ok(Operation::F64Neg),
			0x9b => Ok(Operation::F64Ceil),
			0x9c => Ok(Operation::F64Floor),
			0x9d => Ok(Operation::F64Trunc),
			0x9e => Ok(Operation::F64Nearest),
			0x9f => Ok(Operation::F64Sqrt),
			0xa0 => Ok(Operation::F64Add),
			0xa1 => Ok(Operation::F64Sub),
			0xa2 => Ok(Operation::F64Mul),
			0xa3 => Ok(Operation::F64Div),
			0xa4 => Ok(Operation::F64Min),
			0xa5 => Ok(Operation::F64Max),
			0xa6 => Ok(Operation::F64Copysign),

			// Conversions
			0xa7 => Ok(Operation::I32WrapI64),
			0xa8 => Ok(Operation::I32TruncSF32),
			0xa9 => Ok(Operation::I32TruncUF32),
			0xaa => Ok(Operation::I32TruncSF64),
			0xab => Ok(Operation::I32TruncUF64),
			0xac => Ok(Operation::I64ExtendSI32),
			0xad => Ok(Operation::I64ExtendUI32),
			0xae => Ok(Operation::I64TruncSF32),
			0xaf => Ok(Operation::I64TruncUF32),
			0xb0 => Ok(Operation::I64TruncSF64),
			0xb1 => Ok(Operation::I64TruncUF64),
			0xb2 => Ok(Operation::F32ConvertSI32),
			0xb3 => Ok(Operation::F32ConvertUI32),
			0xb4 => Ok(Operation::F32ConvertSI64),
			0xb5 => Ok(Operation::F32ConvertUI64),
			0xb6 => Ok(Operation::F32DemoteF64),
			0xb7 => Ok(Operation::F64ConvertSI32),
			0xb8 => Ok(Operation::F64ConvertUI32),
			0xb9 => Ok(Operation::F64ConvertSI64),
			0xba => Ok(Operation::F64ConvertUI64),
			0xbb => Ok(Operation::F64PromoteF32),

			// Reinterpretations
			0xbc => Ok(Operation::I32ReinterpretF32),
			0xbd => Ok(Operation::I64ReinterpretF64),
			0xbe => Ok(Operation::F32ReinterpretI32),
			0xbf => Ok(Operation::F64ReinterpretI64),

			_ => Err(ParseError::CustomError("Unknown opcode".to_string()))
		}
	}
}

impl BlockType {
	pub fn parse(reader: &mut Read, module: &ParseModule) -> Result<BlockType, ParseError> {
		let byte = reader.bytes().read_varint(7).unwrap();
		if let Ok(value_type) = ValueType::get(byte) {
			Ok(BlockType::Value(value_type))
		} else if byte == -0x40 {
			Ok(BlockType::Empty)
		} else {
			Err(ParseError::CustomError("Block type wasn't valid".to_string()))
		}
	}
}

impl BranchTable {
	pub fn parse(reader: &mut Read, module: &ParseModule) -> Result<BranchTable, ParseError> {
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

impl MemoryImmediate {
	pub fn parse(reader: &mut Read, module: &ParseModule) -> Result<MemoryImmediate, ParseError> {
		let flags = reader.bytes().read_varuint(32).unwrap() as u32;
		let offset = reader.bytes().read_varuint(32).unwrap() as u32;
		Ok(MemoryImmediate{flags, offset})
	}
}