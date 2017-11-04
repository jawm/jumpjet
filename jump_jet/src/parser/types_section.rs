use std::io::Read;

use parser::leb::unsigned;
use parser::ParseError;

use tree::section::Section;
use tree::language_types::LanguageType;
use tree::language_types::ValueType;
use tree::types::TypeEntry;
use tree::types::TypeSection;


pub fn parse(reader: &mut Read) -> Result<Box<Section>, ParseError> {
    let count = unsigned(&mut reader.bytes())?;
    let mut entries: Vec<TypeEntry> = vec![];
    for entry in 0..count {
        let form = LanguageType::parse(reader)?;
        if form != LanguageType::func {
            return Err(ParseError::InvalidTypeForm)
        }
        let param_count = unsigned(&mut reader.bytes())?;
        let mut params: Vec<ValueType> = vec![];
        for param_index in 0..param_count {
            params.push(ValueType::parse(reader)?);
        }
        let return_count =  unsigned(&mut reader.bytes())?;
        let mut returns: Vec<ValueType> = vec![];
        if return_count > 1 {
            return Err(ParseError::TooManyReturns);
        } else if return_count == 0 {

        } else {
            returns.push(ValueType::parse(reader)?);
        }
        entries.push(TypeEntry{form: form, params: params, returns: returns});
    }
    Ok(Box::new(TypeSection{types:entries}))
}