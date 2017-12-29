use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use tree::Module;

pub fn parse(reader: &mut Read, module: &mut Module) -> Result<(), ParseError> {
    //Err(ParseError::CustomError("not implemented yet".to_string()))
    let start_function = &module.functions[reader.bytes().read_varuint(32)? as usize];
    module.start_function = Some(start_function.clone());
    Ok(())
}