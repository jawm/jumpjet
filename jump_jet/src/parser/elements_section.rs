use std::collections::HashMap;
use std::io::Read;

use parser::leb::unsigned;
use parser::ParseError;

use tree::Module;
use tree::section::Section;
use tree::elements::ElementSection;
use tree::elements::ElementSegment;

pub fn parse(mut reader: &mut Read, sections: &Module) -> Result<Box<Section>, ParseError> {
    let count = unsigned(&mut reader.bytes())?;
    let mut entries = vec![];
    for entry in 0..count {
            let index = unsigned(&mut reader.bytes())?;
            let init_expr = parse_expression(&mut reader);
            let num_elem = unsigned(&mut reader.bytes())?;
            let mut elements = vec![];
            for element in 0..num_elem {
                elements.push(unsigned(&mut reader.bytes())?);
            }
            entries.push(ElementSegment{
                index: index,
                offset: 0,
                elements: elements
            })
    }
    Ok(Box::new(ElementSection{
        entries: entries
    }))
}

struct Code {
    variables: Vec<i8>,
    expressions: Vec<i8>
}

// this is temporary until I actually start parsing expresssions properly.
fn parse_expression(reader: &mut Read) {
    loop {
        let byte = reader.bytes().next().unwrap().unwrap();
        if byte == 0x0b {
            break;
        }
    }
}