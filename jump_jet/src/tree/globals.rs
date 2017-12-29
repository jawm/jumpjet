use super::language_types::GlobalType;
use tree::section::Section;

pub struct GlobalSection {
	pub entries: Vec<GlobalEntry>
}

impl Section for GlobalSection {}

pub struct GlobalEntry {
	pub data_type: GlobalType,
	pub initial: i64// TODO figure out init_expr
}

#[derive(Debug)]
pub struct Global {
    pub constraints: GlobalType,
    pub value: i64 // TODO should be an init_expr
}