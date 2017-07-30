use super::language_types::{LanguageType, ValueType};

pub struct TypeSection {
	types: Vec<TypeEntry>
}

pub struct TypeEntry {
	form: LanguageType, //almost certainly 'func'
	params: Vec<ValueType>,
	returns: Vec<ValueType> // for now, at most length 1
}