use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use parse_tree::functions::Function;
use parse_tree::functions::FuncBody;
use parse_tree::ParseModule;
use parse_tree::types::TypeDefinition;

pub fn parse(reader: &mut Read, module: &mut ParseModule) -> Result<(), ParseError> {
    debug!("Parsing functions section");
    let mut bytes = reader.bytes();
    let count = bytes.read_varuint(32).unwrap();

    for _ in 0..count {
        let index = bytes.read_varuint(32).unwrap();
        if let Some(&TypeDefinition::Func(ref signature)) = module.types.get(index as usize) {
            module.function_signatures.push(index as usize);
        } else {
            return Err(ParseError::CustomError("The type doesn't exist or isn't a function signature".to_string()));
        }
    }
    Ok(())
}