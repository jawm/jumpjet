use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;
use parser::utils;

use parse_tree::language_types::ExternalKind;
use parse_tree::ParseModule;

pub fn parse(reader: &mut Read, module: &mut ParseModule) -> Result<(), ParseError> {
    debug!("Parsing exports section");
    let count = reader.bytes().read_varuint(32).unwrap();
    for _ in 0..count {
        let field = utils::read_string(reader)?;
        let kind = ExternalKind::parse(reader)?;
        module.exports.insert(field, kind);
    }
    Ok(())
}