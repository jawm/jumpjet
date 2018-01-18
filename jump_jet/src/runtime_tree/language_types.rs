use runtime_tree::RuntimeModule;

pub enum ExternalKindInstance {
    Function(Box<Fn(&RuntimeModule, Vec<ValueTypeProvider>)->Vec<ValueTypeProvider>>),
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