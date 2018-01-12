use parse_tree::language_types::GlobalType;
use parse_tree::language_types::InitExpression;

#[derive(Debug)]
pub struct Global {
    pub constraints: GlobalType,
    pub value: InitExpression
}