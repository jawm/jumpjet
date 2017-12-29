use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;
use parser::utils;

use tree::data::DataSection;
use tree::data::DataSegment;
use tree::Module;
use tree::section::Section;

// TODO finish implementing.
pub fn parse(reader: &mut Read, module: &mut Module) -> Result<(), ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    for _ in 0..count {
        let index = reader.bytes().read_varuint(32).unwrap() as usize;
        if index != 0 {
            return Err(ParseError::CustomError("Data index must be 0 in wasm 1.0".to_string()));
        }
        let offset = utils::swallow_expr(&mut reader.bytes()) as usize; // TODO FIGURE OUT PARSING OF EXPRESSIONS - SHOULD BE I32 INITIALISER
        let size = reader.bytes().read_varuint(32).unwrap();
        let mut data = vec![];
        reader.take(size).read_to_end(&mut data);

        let memory = &mut module.memories[index];
        if let Some(max) = memory.limits.maximum {
            if offset + size as usize > max as usize {
                // TODO not sure of correct behaviour here...
                return Err(ParseError::CustomError("adding to big memory".to_string()));
            }
        }
        memory.values.splice(offset..offset+size as usize-1, data);
    }
    Ok(())
}