use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;
use parser::utils;

use tree::code::CodeSection;
use tree::code::FunctionBody;
use tree::language_types::ValueType;
use tree::Module;
use tree::section::Section;

// TODO finish implementing.
pub fn parse(mut reader: &mut Read, _module: &mut Module) -> Result<(), ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    let mut function_bodies = vec![];
    for _ in 0..count {
        let body_size = reader.bytes().read_varuint(32).unwrap();
        let local_count = reader.bytes().read_varuint(32).unwrap();
        let mut locals = vec![];
        for _ in 0..local_count {
            let local_quantity = reader.bytes().read_varuint(32).unwrap();
            let local_type = ValueType::parse(&mut reader.bytes())?;
            locals.push((local_quantity, local_type));
        }
        utils::swallow_expr(&mut reader.bytes());
        function_bodies.push(FunctionBody {
            locals,
            code: vec![]
        });
    }
    Ok(())
}