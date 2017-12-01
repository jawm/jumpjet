use tree::section::Section;
use super::language_types::{LanguageType, ValueType};
use std::clone::Clone;

#[derive(Debug)]
pub struct TypeSection {
    pub types: Vec<TypeEntry>,
}

impl Section for TypeSection {}

#[derive(Debug)]
#[derive(Clone)]
pub struct TypeEntry {
    pub form: Box<LanguageType>, //almost certainly 'func'
    pub params: Vec<Box<ValueType>>,
    pub returns: Vec<Box<ValueType>>, // for now, at most length 1
}
