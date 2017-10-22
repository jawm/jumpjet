use tree::section::Section;
use super::language_types::{LanguageType, ValueType};

pub struct TypeSection {
	pub types: Vec<TypeEntry>
}

impl Section for TypeSection {}

pub struct TypeEntry {
	pub form: LanguageType, //almost certainly 'func'
	pub params: Vec<ValueType>,
	pub returns: Vec<ValueType> // for now, at most length 1
}