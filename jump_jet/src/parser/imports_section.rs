use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;
use parser::utils::read_string;

use tree::imports::ImportEntry;
use tree::imports::ImportSection;
use tree::language_types::ExternalKind;
use tree::Module;
use tree::section::Section;


pub fn parse(reader: &mut Read, module: &mut Module) -> Result<(), ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    let mut entries = vec![];
    for entry in 0..count {
        let module_name = read_string(reader)?;
        let field = read_string(reader)?;
        let kind = ExternalKind::parse(reader, module)?;
        entries.push(ImportEntry {
            module: module_name,
            field,
            kind
        });
    }
    Ok(())
}