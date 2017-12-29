use tree::section::Section;
use super::language_types::{LanguageType, ValueType};
use tree::functions::FuncSignature;

#[derive(Debug)]
pub struct TypeSection {
    pub types: Vec<TypeEntry>,
}

impl Section for TypeSection {}

#[derive(Debug)]
#[derive(Clone)]
pub struct TypeEntry {
    pub form: LanguageType, //almost certainly 'func'
    pub params: Vec<ValueType>,
    pub returns: Vec<ValueType>, // for now, at most length 1
}

#[derive(Debug)]
pub enum TypeDefinition {
    func(FuncSignature)
}
pub trait TypeInstance {}

