use tree::language_types::LanguageType;
use tree::language_types::TableType;
use tree::language_types::ResizableLimits;
use tree::section::Section;

#[derive(Debug)]
pub struct TableSection {
    pub entries: Vec<TableType>,
}

impl Section for TableSection {}

#[derive(Debug)]
pub enum Table {
    AnyFunc {
        limits: ResizableLimits,
        values: Vec<usize>,
    }
}