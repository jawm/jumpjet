use tree::section::Section;
use super::language_types::{Operation, ValueType};

pub struct CodeSection {
	function_bodies: Vec<FunctionBody>
}

impl Section for CodeSection {}

pub struct FunctionBody {
	locals: Vec<ValueType>,
	code: Vec<Operation>
}