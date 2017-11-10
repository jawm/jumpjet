use std::collections::HashMap;
use std::io::Read;

use parser::byteorder::ReadBytesExt;
use parser::leb::unsigned;
use parser::leb::signed;
use parser::ParseError;

use tree::section::Section;
use tree::code::CodeSection;
use tree::code::FunctionBody;
use tree::language_types::ValueType;

// TODO finish implementing.
pub fn parse(mut reader: &mut Read, sections: &HashMap<u64, Box<Section>>) -> Result<Box<Section>, ParseError> {
    let count = unsigned(&mut reader.bytes())?;
    let mut entries = vec![];
    for entry in 0..count {
        let body_size = unsigned(&mut reader.bytes())?;
        let local_count = unsigned(&mut reader.bytes())?;
        let mut locals = vec![];
        for i in 0..local_count {
            let local_quantity = unsigned(&mut reader.bytes())?;
            let local_type = ValueType::parse(&mut reader)?;
            locals.push(local_type);
        }
        println!("locals {:?} ", locals);
        parse_expression(&mut reader);
        entries.push(FunctionBody {
            locals: locals,
            code: vec![]
        });
    }
    println!("returning");
    Ok(Box::new(CodeSection{
        function_bodies: entries
    }))
}

// this is temporary until I actually start parsing expresssions properly.
fn parse_expression(reader: &mut Read) {
    loop {
        let byte = reader.bytes().next().unwrap().unwrap();
        print!("{:x} ", byte);
        if byte == 0x0b {
            break;
        }
    }
    println!("break");
}