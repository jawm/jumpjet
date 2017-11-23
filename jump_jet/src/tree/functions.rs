use tree::section::Section;
use tree::types::TypeEntry;

#[derive(Debug)]
pub struct FunctionSection {
    pub functions: Vec<TypeEntry>, // This might be better to use actual struct rather than an index
}

impl Section for FunctionSection {}
