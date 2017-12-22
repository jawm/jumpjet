use tree::section::Section;
use tree::types::TypeEntry;

pub struct StartSection {
	pub start_function: TypeEntry
}

impl Section for StartSection {}