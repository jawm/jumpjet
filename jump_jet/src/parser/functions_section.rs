use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use tree::functions::Function;
use tree::Module;
use tree::types::TypeDefinition;

pub fn parse(reader: &mut Read, module: &mut Module) -> Result<(), ParseError> {
    let mut bytes = reader.bytes();
    let count = bytes.read_varuint(32).unwrap();

    for _ in 0..count {
        let index = bytes.read_varuint(32).unwrap();
        if let Some(&TypeDefinition::Func(ref signature)) = module.types.get(index as usize) {
            module.functions.push(Function {signature: signature.clone()});
        } else {
            return Err(ParseError::CustomError("The type doesn't exist or isn't a function signature".to_string()));
        }
    }
    Ok(())
}