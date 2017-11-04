use std::io::Read;

use parser::leb::unsigned;
use parser::ParseError;

use tree::section::Section;
use tree::language_types::LanguageType;
use tree::language_types::ValueType;
use tree::types::TypeEntry;
use tree::functions::FunctionSection;


pub fn parse(reader: &mut Read) -> Result<Box<Section>, ParseError> {
    let count = unsigned(&mut reader.bytes())?;
    let mut entries: Vec<u64> = vec![];
    for entry in 0..count {
        let index = unsigned(&mut reader.bytes())?;
        entries.push(index);
    }
    Ok(Box::new(FunctionSection{
        functions: entries
    }))
}