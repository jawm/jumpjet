use tree::language_types::MemoryType;
use tree::language_types::ResizableLimits;
use tree::section::Section;

pub struct MemorySection {
	pub entries: Vec<MemoryType>
}

impl Section for MemorySection {}

#[derive(Debug)]
pub struct Memory {
    pub limits: ResizableLimits,
    pub values: Vec<u8>
}