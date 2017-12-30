use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use tree::language_types::Operation;
use tree::language_types::ValueType;
use tree::Module;

// TODO finish implementing.
pub fn parse(reader: &mut Read, module: &mut Module) -> Result<(), ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    for index in 0..count {
        let _body_size = reader.bytes().read_varuint(32).unwrap();
        let local_count = reader.bytes().read_varuint(32).unwrap();
        let mut locals = vec![];
        for _ in 0..local_count {
            let local_quantity = reader.bytes().read_varuint(32).unwrap() as usize;
            let local_type = ValueType::parse(&mut reader.bytes())?;
            let mut l = vec![local_type; local_quantity];
            locals.append(&mut l);
        }
        match Operation::parse_multiple(reader, module) {
            Ok(code) => {
                module.functions[index as usize].body.locals = locals;
                module.functions[index as usize].body.code = code;
            },
            Err(e) => {return Err(e)},
        }
    }
    Ok(())
}