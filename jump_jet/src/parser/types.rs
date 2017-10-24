extern crate byteorder;
extern crate leb;
use self::byteorder::ReadBytesExt;
use self::byteorder::LittleEndian;
use self::leb::signed;
use self::leb::unsigned;

use std::io::Read;
use super::super::tree::types::*;
use super::super::tree::language_types::*;
use super::SectionParse;
use super::ParseError;
use super::super::tree::section::*;
use super::language_types;

impl SectionParse for TypeSection {

	fn parse<T: Read>(reader: T) -> Result<Section, ParseError> {
		// let count = unsigned(&mut reader.bytes())?;
		// let mut entries: Vec<TypeEntry> = vec![];
		// for entry in 0..count {
		// 	let form = language_types::parse_type(&mut reader)?;

		// 	if let Type::func = form {
		// 		let param_count = unsigned(&mut reader.bytes())?;
		// 		let mut params: Vec<ValueType> = vec![];
		// 		for param_index in 0..param_count {
		// 			params.push(language_types::parse_value_type(&mut reader)?);
		// 		}
		// 		let return_count =  unsigned(&mut reader.bytes())?;
		// 		let mut returns: Vec<ValueType> = vec![];
		// 		if (return_count > 1) {
		// 			return Err(ParseError::TooManyReturns);
		// 		} else if (return_count == 0) {

		// 		} else {
		// 			returns.push(language_types::parse_value_type(&mut reader)?);
		// 		}
		// 		entries.push(TypeEntry{form: form, params: params, returns: returns});
		// 	} else {
		// 		return Err(ParseError::InvalidTypeForm)
		// 	}

			
		// }
		// Ok(Section::Type(TypeSection{types:entries}))
		Err(ParseError::InvalidTypeForm)
	}
}