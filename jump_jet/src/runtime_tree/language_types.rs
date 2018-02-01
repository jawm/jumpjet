use runtime_tree::RuntimeModule;
use runtime_tree::Func;

pub enum ExternalKindInstance {
    Function(Func),
    Table(usize),
    Memory(usize),
    Global(usize),
}

#[derive(Debug, Clone)]
pub enum ValueTypeProvider {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}