use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use tree::Module;
use tree::section::Section;
use tree::start::StartSection;
use tree::types::TypeSection;

pub fn parse(reader: &mut Read, module: &Module) -> Result<Box<Section>, ParseError> {
    let start_function = module.get_section::<TypeSection>(1).unwrap().types[
        reader.bytes().read_varuint(32)? as usize
    ];
    Ok(Box::new(StartSection{start_function}))
}