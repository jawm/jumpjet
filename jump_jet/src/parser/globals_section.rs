use std::io::Read;

use parser;
use parser::leb::ReadLEB;
use parser::ParseError;

use tree::globals::GlobalEntry;
use tree::globals::GlobalSection;
use tree::language_types::GlobalType;
use tree::Module;
use tree::section::Section;

pub fn parse(reader: &mut Read, module: &mut Module) -> Result<(), ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    let mut entries = vec![];
    for entry in 0..count {
        let data_type = GlobalType::parse(reader)?;
        entries.push(GlobalEntry{
            data_type,
            initial: parser::utils::swallow_expr(&mut reader.bytes()) // TODO replace this with a `init_expr`
        });
    }
    Ok(())
}