use super::language_types::ResizableLimits;
use tree::section::Section;

pub struct MemorySection {
	pub entries: Vec<ResizableLimits>
}

impl Section for MemorySection {}