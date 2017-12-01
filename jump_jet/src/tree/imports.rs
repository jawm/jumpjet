use super::language_types::ExternalKind;
use tree::section::Section;

pub struct ImportSection {
	pub entries: Vec<ImportEntry>
}

impl Section for ImportSection {}

pub struct ImportEntry {
	pub module: String,
	pub field: String,
	pub kind: ExternalKind
}