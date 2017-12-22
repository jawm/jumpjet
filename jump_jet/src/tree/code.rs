use tree::section::Section;
use super::language_types::{Operation, ValueType};

#[derive(Debug)]
pub struct CodeSection {
    pub function_bodies: Vec<FunctionBody>,
}
#[derive(Debug)]
pub struct FunctionBody {
    pub locals: Vec<(u64, ValueType)>,
    pub code: Vec<Operation>,
}

impl Section for CodeSection {}
