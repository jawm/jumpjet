use parse_tree::functions::FuncSignature;

#[derive(Debug)]
pub enum TypeDefinition {
    Func(FuncSignature)
}