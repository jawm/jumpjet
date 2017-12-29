use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use tree::language_types::LanguageType;
use tree::language_types::TableType;
use tree::Module;
use tree::section::Section;
use tree::tables::Table;

pub fn parse(reader: &mut Read, module: &mut Module) -> Result<(), ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    for _ in 0..count {
        let constraints = TableType::parse(reader)?;
        match constraints.elem_type {
            AnyFunc => {
                let capacity = constraints.limits.maximum.unwrap_or(constraints.limits.initial) as usize;
                let mut vec = Vec::with_capacity(capacity);
                vec.append(&mut vec![0;constraints.limits.initial as usize]);
                module.tables.push(Table::AnyFunc {
                    limits: constraints.limits,
                    values: vec
                });
            },
            _ => return Err(ParseError::CustomError("Only table of <anyfunc> is supported in 1.0".to_string()))
        }
    }
    Ok(())
}