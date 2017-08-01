extern crate byteorder;
extern crate leb;
use self::byteorder::ReadBytesExt;
use self::byteorder::LittleEndian;
use self::leb::signed;
use self::leb::unsigned;

use std::io;
use std::io::Read;
use std::collections::HashMap;

use super::tree::Module;
use super::tree::section::Section;

const MAGIC_NUMBER: u32 = 0x6d736100;


pub trait SectionParse {
	fn parse<T: Read>(reader: T) -> Result<Section, ParseError>;
}

#[derive(Debug)]
pub enum ParseError {
	WrongMagicNumber,
	UnknownSectionId,
	UnsupportedModuleVersion,
	SectionLengthWrong,
	Io(io::Error)
}

impl From<io::Error> for ParseError {
	fn from(err: io::Error) -> ParseError {
		ParseError::Io(err)
	}
}

impl Module {

	pub fn parse<T: Read>(reader: &mut T) -> Result<Module, ParseError> {
		let magic_number = reader.read_u32::<LittleEndian>()?;
		if magic_number != MAGIC_NUMBER {
			return Err(ParseError::WrongMagicNumber);
		}
		let version = reader.read_u32::<LittleEndian>()?;
		if version != 1 {
			return Err(ParseError::UnsupportedModuleVersion);
		} else {
			let sections = Module::parse_sections(reader)?;
			return Ok(Module{sections:sections, version:version});
		}
	}

	fn parse_sections<T: Read>(reader: &mut T) -> Result<Vec<Section>, ParseError> {
		Ok(vec![])
	}

	fn parse_section<T: Read>(reader: &mut T) -> Result<Section, ParseError> {
		let id = unsigned(&mut reader.bytes())?;
		let length = unsigned(&mut reader.bytes())?;
		let mut subreader = reader.take(length);
		return match id {
			1 => Module::read_section_types(&mut subreader),
			2 => Module::read_section_imports(&mut subreader),
			3 => Module::read_section_functions(&mut subreader),
			4 => Module::read_section_table(&mut subreader),
			5 => Module::read_section_memory(&mut subreader),
			6 => Module::read_section_global(&mut subreader),
			7 => Module::read_section_exports(&mut subreader),
			8 => Module::read_section_start(&mut subreader),
			9 => Module::read_section_elements(&mut subreader),
			10=> Module::read_section_code(&mut subreader),
			11=> Module::read_section_data(&mut subreader),
			_ => Err(ParseError::UnknownSectionId)
		}
	}

	fn read_section_types<T: Read>(reader: &mut T) -> Result<Section, ParseError> {

		// Ok(Section::Type())
    	Err(ParseError::WrongMagicNumber)
    }

    fn read_section_imports<T: Read>(reader: &mut T) -> Result<Section, ParseError> {
    	Err(ParseError::WrongMagicNumber)
    }

    fn read_section_functions<T: Read>(reader: &mut T) -> Result<Section, ParseError> {
    	Err(ParseError::WrongMagicNumber)
    }

    fn read_section_table<T: Read>(reader: &mut T) -> Result<Section, ParseError> {
    	Err(ParseError::WrongMagicNumber)
    }

    fn read_section_memory<T: Read>(reader: &mut T) -> Result<Section, ParseError> {
    	Err(ParseError::WrongMagicNumber)
    }

    fn read_section_global<T: Read>(reader: &mut T) -> Result<Section, ParseError> {
    	Err(ParseError::WrongMagicNumber)
    }

    fn read_section_exports<T: Read>(reader: &mut T) -> Result<Section, ParseError> {
    	Err(ParseError::WrongMagicNumber)
    }

    fn read_section_start<T: Read>(reader: &mut T) -> Result<Section, ParseError> {
    	Err(ParseError::WrongMagicNumber)
    }

    fn read_section_elements<T: Read>(reader: &mut T) -> Result<Section, ParseError> {
    	Err(ParseError::WrongMagicNumber)
    }

    fn read_section_code<T: Read>(reader: &mut T) -> Result<Section, ParseError> {
    	Err(ParseError::WrongMagicNumber)
    }

    fn read_section_data<T: Read>(reader: &mut T) -> Result<Section, ParseError> {
    	Err(ParseError::WrongMagicNumber)
    }
}