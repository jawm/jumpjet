use std::io::Read;

use parser::leb::unsigned;
use parser::ParseError;

use tree::Module;
use tree::section::Section;
use tree::types::TypeSection;
use tree::functions::FunctionSection;


pub fn parse(reader: &mut Read, module: &Module) -> Result<Box<Section>, ParseError> {
	println!("should be true: {}", module.get_section::<TypeSection>(1).is_some());
    let count = unsigned(&mut reader.bytes())?;
    let mut entries = vec![];
    for entry in 0..count {
        let index = unsigned(&mut reader.bytes())?;
        match module.get_section::<TypeSection>(0) {
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