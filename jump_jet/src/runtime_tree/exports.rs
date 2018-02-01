use runtime_tree::language_types::ExternalKindInstance;
use runtime_tree::RuntimeModule;
use runtime_tree::ValueTypeProvider;
use runtime_tree::CallFrame;

pub struct ExportObj<'m> {
    pub module: &'m mut RuntimeModule
}
pub trait ExportObject {
    fn call_fn(&mut self, name: &str, args: Vec<ValueTypeProvider>) -> Vec<ValueTypeProvider>;
}
impl<'m> ExportObject for ExportObj<'m> {
    fn call_fn(&mut self, name: &str, args: Vec<ValueTypeProvider>) -> Vec<ValueTypeProvider> {
        let export = self.module.exports.get(name).unwrap();
        if let ExternalKindInstance::Function(ref i) = *export {
            let mut c = CallFrame {
                types: & self.module.types,
                tables: & self.module.tables,
                globals: &mut self.module.globals,
                memories: &mut self.module.memories,
                functions: & self.module.functions
            };
            return i(&mut c, args);
        } else {
            panic!("export wasn't a function");
        }
    }
}