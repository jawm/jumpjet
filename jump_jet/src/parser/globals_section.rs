use std::io::Read;

use parser;
use parser::leb::ReadLEB;
use parser::ParseError;

use tree::globals::Global;
use tree::language_types::GlobalType;
use tree::Module;

pub fn parse(reader: &mut Read, module: &mut Module) -> Result<(), ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    for _ in 0..count {
        let constraints = GlobalType::parse(reader)?;
        module.globals.push(Global{
            constraints,
            value: parser::utils::swallow_expr(&mut reader.bytes()) // TODO replace this with a `init_expr`
        });
    }
    Ok(())
}