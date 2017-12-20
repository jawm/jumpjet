use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;
use parser::utils::read_string;

use tree::language_types::ExternalKind;
use tree::Module;
use tree::section::Section;
use tree::imports::ImportEntry;
use tree::imports::ImportSection;


pub fn parse(reader: &mut Read, module: &Module) -> Result<Box<Section>, ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    let mut entries = vec![];
    for entry in 0..count {
        let module_name = read_string(reader);
        let field_name = read_string(reader);
        let kind = ExternalKind::parse(reader)?;
        entries.push(ImportEntry {
            module: module_name?,
            field: field_name?,
            kind: kind
        });
    }
    Ok(Box::new(ImportSection{
        entries: entries
    }))
}