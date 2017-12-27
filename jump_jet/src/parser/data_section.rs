use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;
use parser::utils;

use tree::data::DataSection;
use tree::data::DataSegment;
use tree::Module;
use tree::section::Section;

// TODO finish implementing.
pub fn parse(reader: &mut Read, _module: &mut Module) -> Result<(), ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    let mut entries = vec![];
    for _ in 0..count {
        let index = reader.bytes().read_varuint(32).unwrap();
        if index != 0 {
            return Err(ParseError::CustomError("Data index must be 0 in wasm 1.0".to_string()));
        }
        let offset = utils::swallow_expr(&mut reader.bytes()); // TODO FIGURE OUT PARSING OF EXPRESSIONS - SHOULD BE I32 INITIALISER
        let size = reader.bytes().read_varuint(32).unwrap();
        let mut data = vec![];
        reader.take(size).read_to_end(&mut data);
        entries.push(DataSegment {
            index,
            offset,
            data
        });
    }
    Ok(())
}