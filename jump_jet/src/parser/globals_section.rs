use std::collections::HashMap;
use std::io::Read;

use parser::leb::unsigned;
use parser::ParseError;

use tree::language_types::GlobalType;
use tree::Module;
use tree::section::Section;
use tree::globals::GlobalEntry;
use tree::globals::GlobalSection;

pub fn parse(reader: &mut Read, module: &Module) -> Result<Box<Section>, ParseError> {
    let count = unsigned(&mut reader.bytes())?;
    let mut entries = vec![];
    for entry in 0..count {
        let data_type = GlobalType::parse(reader)?;
        entries.push(GlobalEntry{
            data_type: data_type,
            initial: 0
        });
    }
    Ok(Box::new(GlobalSection {
        entries: entries
    }))
}