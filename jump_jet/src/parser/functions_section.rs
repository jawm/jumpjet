use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use tree::Module;
use tree::section::Section;
use tree::types::TypeSection;
use tree::functions::FunctionSection;


pub fn parse(reader: &mut Read, module: &Module) -> Result<Box<Section>, ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    let mut entries = vec![];
    for entry in 0..count {
        let mut x = reader.bytes();
        let index = x.read_varuint(32).unwrap();
        match module.get_section::<TypeSection>(1) {
            Some(section) => {
                let signature = &section.types[index as usize];
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