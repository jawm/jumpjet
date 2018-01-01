use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;

use runtime::language_types::ValueTypeInstance;

use tree::language_types::ExternalKind;
use tree::language_types::ValueType;
use tree::Module;

pub trait GetExport {
    fn get_function<'func>(&'func self, export_name: &str, module: &'func Module) -> Result<Box<(Fn(Vec<ValueTypeInstance>) -> Result<Vec<ValueTypeInstance>,&'func Error>) + 'func>, &Error>;
}

pub trait ValueTypeProvider : Debug {
    fn get_value(&self) -> ValueTypeInstance;
}
impl ValueTypeProvider for i32 {
    fn get_value(&self) -> ValueTypeInstance {
        ValueTypeInstance::I32(*self)
    }
}
impl ValueTypeProvider for i64 {
    fn get_value(&self) -> ValueTypeInstance {
        ValueTypeInstance::I64(*self)
    }
}
impl ValueTypeProvider for f32 {
    fn get_value(&self) -> ValueTypeInstance {
        ValueTypeInstance::F32(*self)
    }
}
impl ValueTypeProvider for f64 {
    fn get_value(&self) -> ValueTypeInstance {
        ValueTypeInstance::F64(*self)
    }
}

impl GetExport for HashMap<String, ExternalKind> {
    fn get_function<'func>(&'func self, export_name: &str, module: &'func Module) -> Result<Box<(Fn(Vec<ValueTypeInstance>) -> Result<Vec<ValueTypeInstance>,&'func Error>) + 'func>, &Error> {
        if let Some(&ExternalKind::Function(index)) = self.get(export_name){
            let func = &module.functions[index];
            Ok(Box::new(move |arguments|{
                if let Ok(_) = func.check_arguments(&arguments) {
                    println!("{:?}",func);
                    func.execute(arguments)
                } else {
                    panic!("don't have proper error handling yet ayy");
                }
            }))
        } else {
            panic!("something");
        }
    }
}

#[macro_export]
macro_rules! args {
    ($($x:expr),*) => {
        {
            let mut temp_vec = Vec::new();
            $(temp_vec.push(($x).get_value());)*
            temp_vec
        }
    }
}