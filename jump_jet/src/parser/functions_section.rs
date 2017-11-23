use std::any::Any;
use std::collections::HashMap;
use std::io::Read;

use parser::leb::unsigned;
use parser::ParseError;

use tree::Module;
use tree::section::Section;
use tree::language_types::LanguageType;
use tree::language_types::ValueType;
use tree::types::TypeEntry;
use tree::types::TypeSection;
use tree::functions::FunctionSection;


pub fn parse(reader: &mut Read, module: &Module) -> Result<Box<Section>, ParseError> {
	println!("should be true: {}", module.get_section::<TypeSection>(1).is_some());
    let count = unsigned(&mut reader.bytes())?;
    let mut entries = vec![];
    for entry in 0..count {
        let index = unsigned(&mut reader.bytes())?;
        //let function = sections.get(&1).expect("didn't exist");
        //println!("{:?}", function.downcast::<TypeSection>());
        entries.push(index);
    }
    Ok(Box::new(FunctionSection{
        functions: vec![]
    }))
}