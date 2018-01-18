use runtime_tree::language_types::ExternalKindInstance;
use runtime_tree::RuntimeModule;
use runtime_tree::ValueTypeProvider;


pub struct ExportObj<'m> {
    pub module: &'m RuntimeModule
}
pub trait ExportObject {
    fn call_fn(&self, name: &str, args: Vec<ValueTypeProvider>) -> Vec<ValueTypeProvider>;
}
impl<'m> ExportObject for ExportObj<'m> {
    fn call_fn(&self, name: &str, args: Vec<ValueTypeProvider>) -> Vec<ValueTypeProvider> {
        let export = self.module.exports.get(name).unwrap();
        if let ExternalKindInstance::Function(ref i) = *export {
            return i(self.module, args);
        } else {
            panic!("export wasn't a function");
        }
    }
}