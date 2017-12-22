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

pub fn parse(reader: &mut Read, module: &Module) -> Result<Box<Section>, ParseError> {
    let bytes = &mut reader.bytes();
    let count = bytes.read_varuint(32).unwrap();
    let mut types: Vec<TypeEntry> = vec![];
    for entry in 0..count {
        let form = LanguageType::parse(bytes)?;
        if form != LanguageType::func {
            // WASM 1.0 requires all imports to be of type `func`
            return Err(ParseError::InvalidTypeForm);
        }
        let param_count = bytes.read_varuint(32).unwrap();
        let mut params: Vec<ValueType> = vec![];
        for param_index in 0..param_count {
            params.push(ValueType::parse(bytes)?);
        }
        let return_count =  bytes.read_varuint(1).unwrap();
        let mut returns: Vec<ValueType> = vec![];
        if return_count > 1 {
            return Err(ParseError::TooManyReturns);
        } else if return_count == 1 {
            returns.push(ValueType::parse(bytes)?);
        }
        types.push(TypeEntry{form, params, returns});
    }
    Ok(Box::new(TypeSection{types}))
}


impl Parse for TypeSection {
    fn parse(_reader: &mut Read, _module: &Module) -> Result<Box<Section>, ParseError> {
        Err(ParseError::WrongMagicNumber)
    }
}