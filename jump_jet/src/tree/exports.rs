use super::language_types::ExternalKind;
use tree::section::Section;

#[derive(Debug)]
pub struct ExportSection {
    pub entries: Vec<ExportEntry>,
}

#[derive(Debug)]
pub struct ExportEntry {
    pub field: String,
    pub kind: ExternalKind,
}

impl Section for ExportSection {}
