use parse_tree::language_types::Operation;
use parse_tree::language_types::ValueType;

#[derive(Clone, Debug)]
pub struct Function {
    pub signature: FuncSignature,
    pub body: FuncBody,
}

#[derive(Clone, Debug)]
pub struct FuncSignature {
    pub parameters: Vec<ValueType>,
    pub returns: Vec<ValueType>,
}

#[derive(Clone, Debug)]
pub struct FuncBody {
    pub locals: Vec<ValueType>,
    pub code: Vec<Operation>,
}

impl FuncBody {
    pub fn new() -> Self {
        FuncBody {
            locals: vec![],
            code: vec![],
        }
    }
}