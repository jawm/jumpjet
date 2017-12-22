use super::language_types::MemoryType;
use tree::section::Section;

pub struct MemorySection {
	pub entries: Vec<MemoryType>
}

impl Section for MemorySection {}