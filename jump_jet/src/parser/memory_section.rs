use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use tree::language_types::ResizableLimits;
use tree::memory::Memory;
use tree::Module;

pub fn parse(reader: &mut Read, module: &mut Module) -> Result<(), ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    for _ in 0..count {
        let limits = ResizableLimits::parse(reader)?;
        let capacity = limits.maximum.unwrap_or(limits.initial) as usize;
        module.memories.push(Memory{
            limits,
            values: Vec::with_capacity(capacity)
        });
    }
    Ok(())
}