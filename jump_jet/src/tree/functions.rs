use tree::section::Section;

pub struct FunctionSection {
	pub functions: Vec<u64> // This might be better to use actual struct rather than an index
}

impl Section for FunctionSection {}