use std::collections::HashMap;
use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;
use parser::utils::read_string;

use tree::language_types::ExternalKind;
use tree::Module;

pub fn parse(reader: &mut Read, module: &mut Module) -> Result<(), ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    for _ in 0..count {
        let module_name = read_string(reader)?;
        let field = read_string(reader)?;
        let kind = ExternalKind::parse(reader, module)?;

        if !module.imports.contains_key(&module_name) {
            let mut map = HashMap::new();
            map.insert(field, kind);
            module.imports.insert(module_name, map);
        } else if !module.imports.get(&module_name).unwrap().contains_key(&field) {
            module.imports.get_mut(&module_name).unwrap().insert(field, kind);
        } else {
            return Err(ParseError::CustomError("Tried to import an already imported field".to_string()));
        }
    }
    Ok(())
}