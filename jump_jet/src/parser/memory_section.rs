use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use parse_tree::language_types::ResizableLimits;
use parse_tree::memory::Memory;
use parse_tree::memory::WASM_PAGE_SIZE;
use parse_tree::ParseModule;

pub fn parse(reader: &mut Read, module: &mut ParseModule) -> Result<(), ParseError> {
    debug!("Parsing memory section");
    let count = reader.bytes().read_varuint(32).unwrap();
    for _ in 0..count {
        let limits = ResizableLimits::parse(reader)?;
        let capacity = limits.maximum.unwrap_or(limits.initial) as usize;
        let values = vec![0;capacity*WASM_PAGE_SIZE];
        module.memories.push(Memory{limits, values});
    }
    Ok(())
}