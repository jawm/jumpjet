use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;
use parser::utils;

use tree::language_types::ExternalKind;
use tree::Module;

pub fn parse(reader: &mut Read, module: &mut Module) -> Result<(), ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    for _ in 0..count {
        let field = utils::read_string(reader)?;
        let kind = ExternalKind::parse(reader)?;
        module.exports.insert(field, kind);
    }
    Ok(())
}