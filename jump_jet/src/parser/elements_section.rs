use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;
use parser::utils;

use tree::elements::ElementSection;
use tree::elements::ElementSegment;
use tree::Module;
use tree::section::Section;
use tree::types::TypeSection;

pub fn parse(mut reader: &mut Read, module: &Module) -> Result<Box<Section>, ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    let mut entries = vec![];
    for entry in 0..count {
        let index = reader.bytes().read_varuint(32).unwrap();
        if index != 0 {
            return Err(ParseError::CustomError("WASM 1.0 only allows 1 table".to_string()));
        }
        let init_expr = utils::swallow_expr(&mut reader.bytes());
        let num_elem = reader.bytes().read_varuint(32).unwrap();
        let mut elements = vec![];
        for _ in 0..num_elem {
            // elements.push(
            //     module.get_section::<TypeSection>(1).unwrap().types
            //         .get(reader.bytes().read_varuint(32).unwrap() as usize).unwrap()
            // );
        }
        entries.push(ElementSegment{
            index,
            offset: init_expr,
            elements
        })
    }
    Ok(Box::new(ElementSection{entries}))
}