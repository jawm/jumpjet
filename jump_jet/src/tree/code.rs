use super::language_types::{Operation, ValueType};

pub struct CodeSection {
	function_bodies: Vec<FunctionBody>
}

pub struct FunctionBody {
	locals: Vec<ValueType>,
	code: Vec<Operation>
}