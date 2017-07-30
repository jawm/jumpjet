use super::language_types::ExternalKind;

pub struct ExportSection {
	entries: Vec<ExportEntry>
}

pub struct ExportEntry {
	field: String,
	kind: ExternalKind,
	index: u64
}