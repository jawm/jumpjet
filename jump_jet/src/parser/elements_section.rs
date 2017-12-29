use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;
use parser::utils;

use tree::Module;
use tree::tables::Table;

pub fn parse(reader: &mut Read, module: &mut Module) -> Result<(), ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    for _ in 0..count {
        let index = reader.bytes().read_varuint(32).unwrap() as usize;
        if index != 0 {
            return Err(ParseError::CustomError("WASM 1.0 only allows 1 table".to_string()));
        }
        let init_expr = utils::swallow_expr(&mut reader.bytes()) as usize;
        let num_elem = reader.bytes().read_varuint(32).unwrap() as usize;
        let mut elements = vec![];
        for _ in 0..num_elem {
            let item = reader.bytes().read_varuint(32).unwrap() as usize;
            elements.push(item);
        }
        match module.tables[index] {
            Table::AnyFunc {ref mut values, ref limits} => {
                if let Some(max) = limits.maximum {
                    if init_expr + num_elem > max as usize {
                        // TODO check for the proper behaviour on what to do here... might want to keep adding items up to the end
                        return Err(ParseError::CustomError("Attempted to add values to late into table... ".to_string()));
                    }
                }
                values.splice(init_expr..init_expr+num_elem, elements);
            },
        }
    }
    Ok(())
}