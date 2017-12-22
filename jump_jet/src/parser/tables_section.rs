use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use tree::language_types::TableType;
use tree::Module;
use tree::section::Section;
use tree::tables::TableSection;

pub fn parse(reader: &mut Read, module: &Module) -> Result<Box<Section>, ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    let mut entries = vec![];
    for entry in 0..count {
        entries.push(TableType::parse(reader)?);
    }
    Ok(Box::new(TableSection{entries}))
}