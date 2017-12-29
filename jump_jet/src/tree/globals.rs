use super::language_types::GlobalType;

#[derive(Debug)]
pub struct Global {
    pub constraints: GlobalType,
    pub value: i64 // TODO should be an init_expr
}