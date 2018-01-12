use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use parse_tree::language_types::LanguageType;
use parse_tree::language_types::TableType;
use parse_tree::ParseModule;
use parse_tree::tables::Table;

pub fn parse(reader: &mut Read, module: &mut ParseModule) -> Result<(), ParseError> {
    debug!("Parsing tables section");
    let count = reader.bytes().read_varuint(32).unwrap();
    for _ in 0..count {
        let constraints = TableType::parse(reader)?;
        match constraints.elem_type {
            LanguageType::Anyfunc => {
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