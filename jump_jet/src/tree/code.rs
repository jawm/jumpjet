use super::language_types::{Operation, ValueType};

#[derive(Debug)]
pub struct FunctionBody {
    pub locals: Vec<(u64, ValueType)>,
    pub code: Vec<Operation>,
}