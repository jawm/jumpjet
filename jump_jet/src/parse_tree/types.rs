use parse_tree::functions::FuncSignature;

#[derive(Debug, Clone)]
pub enum TypeDefinition {
    Func(FuncSignature)
}