use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use parse_tree::language_types::InitExpression;
use parse_tree::ParseModule;
use parse_tree::tables::Table;

pub fn parse(reader: &mut Read, module: &mut ParseModule) -> Result<(), ParseError> {
    debug!("Parsing elements section");
    let count = reader.bytes().read_varuint(32).unwrap();
    for _ in 0..count {
        let index = reader.bytes().read_varuint(32).unwrap() as usize;
        if index != 0 {
            return Err(ParseError::CustomError("WASM 1.0 only allows 1 table".to_string()));
        }

        if let Ok(InitExpression::I32Const(init_expr)) = InitExpression::parse(reader, module) {
            let init  = init_expr as usize;
            let num_elem = reader.bytes().read_varuint(32).unwrap() as usize;
            let mut elements = vec![];
            for _ in 0..num_elem {
                let item = reader.bytes().read_varuint(32).unwrap() as usize;
                elements.push(item);
            }
            match module.tables[index] {
                Table::AnyFunc {ref mut values, ref limits} => {
                    if let Some(max) = limits.maximum {
                        if init + num_elem > max as usize {
                            // TODO check for the proper behaviour on what to do here... might want to keep adding items up to the end
                            return Err(ParseError::CustomError("Attempted to add values to late into table... ".to_string()));
                        }
                    }
                    values.splice(init..init+num_elem, elements);
                },
            }
        } else {
            return Err(ParseError::CustomError("init_expr for elements section must be i32.const".to_string()));
        }
    }
    Ok(())
}