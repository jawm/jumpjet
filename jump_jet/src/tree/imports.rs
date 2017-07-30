use super::language_types::ExternalKind;

pub struct ImportSection {
	entries: Vec<ImportEntry>
}

pub struct ImportEntry {
	module: String,
	field: String,
	kind: ExternalKind
}