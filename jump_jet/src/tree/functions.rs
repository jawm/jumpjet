use tree::language_types::ValueType;
use tree::types::TypeInstance;

#[derive(Clone, Debug)]
pub struct Function {
    pub signature: FuncSignature
}

#[derive(Clone, Debug)]
pub struct FuncSignature {
    pub parameters: Vec<ValueType>,
    pub returns: Vec<ValueType>,
}
impl TypeInstance for FuncSignature {}