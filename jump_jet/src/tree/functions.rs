use tree::section::Section;

use tree::types::TypeEntry;
use tree::language_types::ValueType;

#[derive(Debug)]
pub struct FunctionSection {
    pub functions: Vec<TypeEntry>, // This might be better to use actual struct rather than an index
}

impl Section for FunctionSection {}

#[derive(Clone, Debug)]
pub struct Function {
    pub signature: FuncSignature
}


use tree::types::TypeInstance;


#[derive(Clone, Debug)]
pub struct FuncSignature {
    pub parameters: Vec<ValueType>,
    pub returns: Vec<ValueType>,
}
impl TypeInstance for FuncSignature {}