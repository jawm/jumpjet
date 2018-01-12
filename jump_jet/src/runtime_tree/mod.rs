use std::collections::HashMap;

use parse_tree::language_types::ExternalKind;
use parse_tree::language_types::Operation;
use parse_tree::language_types::ValueType;
use parse_tree::ParseModule;

use parser::ParseError;

use runtime::language_types::ValueTypeInstance;

pub struct RuntimeModule {
    //pub version: u32,
    //pub types: Vec<types::TypeDefinition>,
    //pub imports: HashMap<String, HashMap<String, ExternalKind>>,
    pub functions: Vec<Box<Fn(Vec<ValueTypeProvider>)->Vec<ValueTypeProvider>>>,
    pub tables: Vec<Table>,
    pub memories: Vec<Memory>,
    pub globals: Vec<Global>,
    pub exports: HashMap<String, ExternalKindInstance>,
    pub start_function: Option<usize>,
}

pub enum ExternalKindInstance {
    Function(usize),
    Table(usize),
    Memory(usize),
    Global(usize),
}
pub enum Table {}
pub struct Memory {}
pub enum Global {}

pub trait RuntimeModuleBuilder {
    fn build(&self, imports: HashMap<String, HashMap<String, ExternalKindInstance>>) -> Result<RuntimeModule, ParseError>;
}

impl RuntimeModuleBuilder for ParseModule {
    fn build(&self, imports: HashMap<String, HashMap<String, ExternalKindInstance>>) -> Result<RuntimeModule, ParseError> {
        let mut m = RuntimeModule {
            functions: vec![],
            tables: vec![],
            memories: vec![],
            globals: vec![],
            exports: HashMap::new(),
            start_function: None
        };
        m.build_functions(&self);
        m.build_exports(&self);
        Ok(m)
    }
}

struct ExportObj<'m> {module: &'m RuntimeModule}
pub trait ExportObject {
    fn call_fn(&self, name: &str, args: Vec<ValueTypeProvider>) -> Vec<ValueTypeProvider>;
}
impl<'m> ExportObject for ExportObj<'m> {
    fn call_fn(&self, name: &str, args: Vec<ValueTypeProvider>) -> Vec<ValueTypeProvider> {
        let export = self.module.exports.get(name).unwrap();
        if let ExternalKindInstance::Function(i) = *export {
            return self.module.functions.get(i).unwrap()(args);
        } else {
            panic!("export wasn't a function");
        }
    }
}

impl RuntimeModule {
    pub fn exports<'m>(&'m self) -> Box<ExportObject + 'm> {
        Box::new(ExportObj{module: &self})
    }

    fn build_functions(&mut self, parse_module: &ParseModule) {
        let mut functions: Vec<Box<Fn(Vec<ValueTypeProvider>)->Vec<ValueTypeProvider>>> = vec![];

        for f in &(parse_module.functions) {

            let args_size = f.signature.parameters.len();
            let locals_size = f.body.locals.len();
            let local_space_size = args_size + locals_size;

            let mut locals = Vec::with_capacity(local_space_size);
            locals.append(&mut f.signature.parameters.clone());
            locals.append(&mut f.body.locals.clone());

            let operations = f.body.code.clone();

            functions.push(Box::new(move |args|{
                if args.len() != args_size {
                    panic!("Wrong number of args provided");
                }
                let mut local_space: Vec<ValueTypeProvider> = Vec::with_capacity(local_space_size);
                for (param, arg) in locals.iter().zip(args.iter()) {
                    local_space.push(match *param {
                        ValueType::I32 => {
                            if let ValueTypeProvider::I32(val) = *arg {
                                arg.clone()
                            } else {
                                panic!("wrong argument type provided");
                            }
                        },
                        ValueType::I64 => {
                            if let ValueTypeProvider::I64(val) = *arg {
                                arg.clone()
                            } else {
                                panic!("wrong argument type provided");
                            }
                        },
                        ValueType::F32 => {
                            if let ValueTypeProvider::F32(val) = *arg {
                                arg.clone()
                            } else {
                                panic!("wrong argument type provided");
                            }
                        },
                        ValueType::F64 => {
                            if let ValueTypeProvider::F64(val) = *arg {
                                arg.clone()
                            } else {
                                panic!("wrong argument type provided");
                            }
                        },
                    });
                }
                for l in &locals[args_size..local_space_size] {
                    local_space.push(match *l {
                        ValueType::I32 => ValueTypeProvider::I32(0),
                        ValueType::I64 => ValueTypeProvider::I64(0),
                        ValueType::F32 => ValueTypeProvider::F32(0.0),
                        ValueType::F64 => ValueTypeProvider::F64(0.0),
                    });
                }

                let mut stack = vec![];
                for operation in &(operations) {
                    println!("operation {:?}", operation);
                    match *operation {
                        Operation::GetLocal(idx) => {
                            stack.push(local_space[idx].clone());
                        },
                        Operation::CallIndirect(idx, _) => {
//                            for vtp in self.functions[idx](vec![]) {
//                                stack.push(vtp);
//                            }
                        },
                        _ => panic!("not supported yet")
                    }
                }

                vec![]
            }));
        }
        self.functions = functions;
    }

    fn build_exports(&mut self, parse_module: &ParseModule) {
        for (key, value) in parse_module.exports.iter() {
            match *value {
                ExternalKind::Function(i) => self.exports.insert(key.clone(), ExternalKindInstance::Function(i)),
                _ => panic!("not suppeasdf")
            };
        }
    }
}

impl ExternalKindInstance {
    pub fn call_function(export_name: &str) -> Box<Fn(Vec<ValueTypeProvider>)->Vec<ValueTypeProvider>> {
        panic!("")
    }
}


pub trait I32Provider {
    fn get_value(&self) -> i32;
}

impl I32Provider for i32 {
    fn get_value(&self) -> i32 {
        *self
    }
}

trait Provider {

}

#[derive(Clone)]
pub enum ValueTypeProvider {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}