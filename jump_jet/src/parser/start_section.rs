use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use parse_tree::ParseModule;

pub fn parse(reader: &mut Read, module: &mut ParseModule) -> Result<(), ParseError> {
    debug!("Parsing start section");
    let start_function = &module.functions[reader.bytes().read_varuint(32)? as usize];
    module.start_function = Some(start_function.clone());
    Ok(())
}