use super::language_types::ExternalKind;
use tree::section::Section;

pub struct ExportSection {
	pub entries: Vec<ExportEntry>
}

pub struct ExportEntry {
	pub field: String,
	pub kind: ExternalKind
}

impl Section for ExportSection {}