use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;
use parser::utils;

use tree::language_types::ExternalKind;
use tree::Module;
use tree::section::Section;
use tree::exports::ExportSection;
use tree::exports::ExportEntry;

pub fn parse(reader: &mut Read, module: &mut Module) -> Result<(), ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    for _ in 0..count {
        let mut field = utils::read_string(reader)?;
        let kind = ExternalKind::by_index(reader, module)?;
        module.exports.insert(field, kind);
    }
    Ok(())
}