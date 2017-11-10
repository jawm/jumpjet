use super::language_types::TableType;

use tree::section::Section;

pub struct TableSection {
	pub entries: Vec<TableType>
}

impl Section for TableSection {}