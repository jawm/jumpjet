use std::collections::HashMap;
use std::io::Read;

use parser::leb::unsigned;
use parser::leb::signed;
use parser::ParseError;

use tree::language_types::ResizableLimits;
use tree::language_types::TableType;
use tree::Module;
use tree::section::Section;
use tree::memory::MemorySection;

pub fn parse(reader: &mut Read, module: &Module) -> Result<Box<Section>, ParseError> {
    let count = unsigned(&mut reader.bytes())?;
    let mut entries = vec![];
    for entry in 0..count {
        let resizable_limits = ResizableLimits::parse(reader)?;
        entries.push(resizable_limits);
    }
    Ok(Box::new(MemorySection {
        entries: entries
    }))
}