use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use parse_tree::functions::FuncBody;
use parse_tree::language_types::Operation;
use parse_tree::language_types::ValueType;
use parse_tree::ParseModule;

// TODO finish implementing.
pub fn parse(reader: &mut Read, module: &mut ParseModule) -> Result<(), ParseError> {
    debug!("Parsing code section");
    let s = "".to_string();
    let count = reader.bytes().read_varuint(32).unwrap();
    for index in 0..count {
        let _body_size = reader.bytes().read_varuint(32).unwrap();
        let local_count = reader.bytes().read_varuint(32).unwrap();
        let mut locals = vec![];
        debug!("about to parse locals");
        for _ in 0..local_count {
            let local_quantity = reader.bytes().read_varuint(32).unwrap() as usize;
            let local_type = ValueType::parse(&mut reader.bytes())?;
            let mut l = vec![local_type; local_quantity];
            locals.append(&mut l);
        }
        match Operation::parse_multiple(reader, module) {
            Ok(code) => {
                module.function_bodies.push(FuncBody{locals, code});
            },
            Err(e) => {return Err(e)},
        }
        debug!("ops parsed");
    }
    Ok(())
}