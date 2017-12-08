use std::io::Read;

use parser::leb::unsigned;
use parser::ParseError;

use tree::Module;
use tree::section::Section;
use tree::types::TypeSection;
use tree::functions::FunctionSection;


pub fn parse(reader: &mut Read, module: &Module) -> Result<Box<Section>, ParseError> {
    let count = unsigned(&mut reader.bytes())?;
    let mut entries = vec![];
    for entry in 0..count {
        let mut x = reader.bytes();
        let index = unsigned(&mut x)?;
        match module.get_section::<TypeSection>(1) {
            Some(section) => {
                let signature = &section.types[0 as usize];
                entries.push(signature.clone());
            },
            None => {
                return Err(ParseError::NonExistantTypeReference);
            }
        }
    }
    Ok(Box::new(FunctionSection{
        functions: entries
    }))
}