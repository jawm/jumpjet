use runtime_tree::language_types::ExternalKindInstance;
use runtime_tree::ModuleInstance;
use runtime_tree::ValueTypeProvider;

pub struct ExportObj<'m> {
    pub module: &'m mut ModuleInstance<'m>
}
pub trait ExportObject {
    fn call_fn(&mut self, name: &str, args: Vec<ValueTypeProvider>) -> Vec<ValueTypeProvider>;
}
impl<'m> ExportObject for ExportObj<'m> {
    fn call_fn(&mut self, name: &str, args: Vec<ValueTypeProvider>) -> Vec<ValueTypeProvider> {
        let export = self.module.exports.get(name).unwrap();
        if let ExternalKindInstance::Function(ref i) = *export {
            return i(&self.module.get_frame(), args);
        } else {
            panic!("export wasn't a function");
        }
    }
}