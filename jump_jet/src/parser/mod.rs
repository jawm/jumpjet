extern crate byteorder;
extern crate leb;
use self::byteorder::ReadBytesExt;
use self::byteorder::LittleEndian;
use self::leb::ReadLEB;

use std::io;
use std::io::Read;
use std::collections::HashMap;

use super::tree::Module;
use super::tree::section::Section;
use super::tree::types::*;

mod utils;

mod language_types;

mod types_section;
mod imports_section;
mod functions_section;
mod tables_section;
mod memory_section;
mod globals_section;
mod exports_section;
mod start_section;
mod elements_section;
mod code_section;
mod data_section;

const MAGIC_NUMBER: u32 = 0x6d736100;

#[derive(Debug)]
pub enum ParseError {
    WrongMagicNumber,
    UnknownSectionId(u64),
    UnsupportedModuleVersion,
    SectionLengthWrong,
    InvalidTypeForm,
    InvalidValueType(i64),
    InvalidLanguageType(i64),
    InvalidExternalKind(u8),
    TooManyReturns,
    Io(io::Error),
    NonExistantTypeReference,
    CustomError(String),
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> ParseError {
        ParseError::Io(err)
    }
}

pub trait Parse {
    fn parse(reader: &mut Read, module: &Module) -> Result<Box<Section>, ParseError>;
}

pub struct ModuleParser {
    sections: HashMap<
        u64,
        Box<
            Fn(&mut Read, &mut Module) -> Result<(), ParseError>
        >
    >
}

impl ModuleParser {

    pub fn default() -> ModuleParser {

        let mut sections: HashMap<u64, Box<Fn(&mut Read, &mut Module) -> Result<(), ParseError>>> = HashMap::new();
        sections.insert(1,  Box::new(types_section::parse));
        sections.insert(2,  Box::new(imports_section::parse));
        sections.insert(3,  Box::new(functions_section::parse));
        sections.insert(4,  Box::new(tables_section::parse));
        sections.insert(5,  Box::new(memory_section::parse));
        sections.insert(6,  Box::new(globals_section::parse));
        sections.insert(7,  Box::new(exports_section::parse));
        sections.insert(8,  Box::new(start_section::parse));
        sections.insert(9,  Box::new(elements_section::parse));
        sections.insert(10, Box::new(code_section::parse));
        sections.insert(11, Box::new(data_section::parse));

        ModuleParser{sections}
    }

    pub fn parse_module<'a, T: Read>(&self, mut reader: T) -> Result<Module<'a>,ParseError> {
        let magic_number = reader.read_u32::<LittleEndian>()?;
        if magic_number != MAGIC_NUMBER {
            return Err(ParseError::WrongMagicNumber)
        }
        let version = reader.read_u32::<LittleEndian>()?;
        if version != 1 {
            return Err(ParseError::UnsupportedModuleVersion)
        } else {
            let mut module = Module {
                version,
                functions: vec![],
                imports: HashMap::new(),
            };
            self.parse_sections(&mut module, &mut reader)?;
            return Ok(module)
        }
    }

    fn parse_sections<T: Read>(&self, module: &mut Module, reader: &mut T) -> Result<(), ParseError> {

        loop {
            let id = match reader.bytes().read_varuint(7) {
                Ok(id) => id,
                Err(_) => break
            };
            println!("parsing section {}", id);
            let section = match self.parse_section(id, reader, module) {
                Ok(section) => section,
                Err(error) => {
                    println!("Failure parsing section {}", id);
                    return Err(error)
                }
            };
            println!("Section parsed {}", id);
            //module.sections.insert(id, section);
        }
        println!("Module parsing complete");
        Ok(())

    }

    fn parse_section<T: Read>(&self, id: u64, reader: &mut T, module: &mut Module) -> Result<(), ParseError> {
        let parser_function = match self.sections.get(&id) {
            Some(func) => func,
            None => return Err(ParseError::UnknownSectionId(id))
        };
        let length = reader.bytes().read_varuint(32).unwrap();
        let mut subreader = reader.take(length);
        match parser_function(&mut subreader, module) {
            Ok(_) => Ok(()),
            Err(e) => Err(e)
        }

    }
}