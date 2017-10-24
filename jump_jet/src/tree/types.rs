use super::language_types::*;

pub struct TypeSection {
	pub types: Vec<TypeEntry>
}

pub struct TypeEntry {
	pub form: Type, //almost certainly 'func'
	pub params: Vec<ValueType>,
	pub returns: Vec<ValueType> // for now, at most length 1
}