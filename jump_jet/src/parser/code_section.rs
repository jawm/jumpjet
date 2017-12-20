use std::collections::HashMap;
use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use tree::Module;
use tree::section::Section;
use tree::code::CodeSection;
use tree::code::FunctionBody;
use tree::language_types::ValueType;

// TODO finish implementing.
pub fn parse(mut reader: &mut Read, sections: &Module) -> Result<Box<Section>, ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    let mut entries = vec![];
    for entry in 0..count {
        let body_size = reader.bytes().read_varuint(32).unwrap();
        let local_count = reader.bytes().read_varuint(32).unwrap();
        let mut locals = vec![];
        for i in 0..local_count {
            let local_quantity = reader.bytes().read_varuint(32).unwrap();
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