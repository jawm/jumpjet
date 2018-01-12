use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use parse_tree::language_types::LanguageType;
use parse_tree::language_types::ValueType;
use parse_tree::ParseModule;

use parse_tree::types::TypeDefinition;

use parse_tree::functions::FuncSignature;

pub fn parse(reader: &mut Read, module: &mut ParseModule) -> Result<(), ParseError> {
    debug!("Parsing types section");
    let bytes = &mut reader.bytes();
    let count = bytes.read_varuint(32).unwrap();
    for _ in 0..count {
        let form = LanguageType::parse(bytes)?;
        match form {
            LanguageType::Func => {
                let parameter_count = bytes.read_varuint(32).unwrap();
                let mut parameters: Vec<ValueType> = vec![];
                for _ in 0..parameter_count {
                    parameters.push(ValueType::parse(bytes)?);
                }
                let return_count =  bytes.read_varuint(1).unwrap();
                let mut returns: Vec<ValueType> = vec![];
                if return_count > 1 {
                    return Err(ParseError::TooManyReturns);
                } else if return_count == 1 {
                    returns.push(ValueType::parse(bytes)?);
                }
                module.types.push(TypeDefinition::Func(FuncSignature {
                    parameters,
                    returns,
                }));
            },
            _ => return Err(ParseError::CustomError("WASM 1.0 requires all defined types to be of type `func`".to_string()))
        }
    }
    Ok(())
}
