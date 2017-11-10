use std::collections::HashMap;
use std::io::Read;

use parser::leb::unsigned;
use parser::leb::signed;
use parser::ParseError;

use tree::language_types::ResizableLimits;
use tree::language_types::TableType;
use tree::section::Section;
use tree::tables::TableSection;

pub fn parse(reader: &mut Read, sections: &HashMap<u64, Box<Section>>) -> Result<Box<Section>, ParseError> {
    let count = unsigned(&mut reader.bytes())?;
    let mut entries = vec![];
    for entry in 0..count {
        let elem_type = signed(&mut reader.bytes())?;
        let flags = unsigned(&mut reader.bytes())?;
        let initial = unsigned(&mut reader.bytes())?;
        let maximum = if flags == 1 {
            Some(unsigned(&mut reader.bytes())?)
        } else {
            None
        };
        entries.push(TableType{
            elemType: elem_type,
            limits: ResizableLimits {initial: initial, maximum:maximum}
        });
    }
    Ok(Box::new(TableSection{
        entries: entries
    }))
}