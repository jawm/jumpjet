use std::io::Read;

use parser::leb::ReadLEB;
use parser::Parse;
use parser::ParseError;

use tree::language_types::LanguageType;
use tree::language_types::ValueType;
use tree::Module;
use tree::section::Section;
use tree::types::TypeEntry;
use tree::types::TypeSection;

use tree::types::TypeDefinition;
use tree::types::TypeInstance;

use tree::functions::FuncSignature;

pub fn parse(reader: &mut Read, module: &mut Module) -> Result<(), ParseError> {
    let bytes = &mut reader.bytes();
    let count = bytes.read_varuint(32).unwrap();
    for _ in 0..count {
        let form = LanguageType::parse(bytes)?;
        match form {
            func => {
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
                module.types.push(TypeDefinition::func(FuncSignature {
                    parameters,
                    returns,
                }));
            },
            _ => return Err(ParseError::CustomError("WASM 1.0 requires all defined types to be of type `func`".to_string()))
        }
    }
    Ok(())
}
