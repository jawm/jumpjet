use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use tree::functions::FunctionSection;
use tree::Module;
use tree::section::Section;
use tree::types::TypeSection;


pub fn parse(reader: &mut Read, module: &Module) -> Result<Box<Section>, ParseError> {
    let mut bytes = reader.bytes();
    let count = bytes.read_varuint(32).unwrap();
    let mut functions = vec![];
    for entry in 0..count {
        let index = bytes.read_varuint(32).unwrap();
        match module.get_section::<TypeSection>(1) {
            Some(section) => {
                let signature = &section.types[index as usize];
                functions.push(signature.clone());
            },
            None => {
                return Err(ParseError::NonExistantTypeReference);
            }
        }
    }
    Ok(Box::new(FunctionSection{functions}))
}