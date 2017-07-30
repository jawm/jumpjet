use super::language_types::GlobalType;

pub struct GlobalSection {
	entries: Vec<GlobalEntry>
}

pub struct GlobalEntry {
	data_type: GlobalType,
	initial: i64// TODO figure out init_expr
}