use std::collections::HashMap;

use parse_tree::language_types::ExternalKind;
use parse_tree::language_types::Operation;
use parse_tree::language_types::ValueType;
use parse_tree::ParseModule;
use parse_tree::tables::Table;
use parse_tree::types::TypeDefinition;

use parser::ParseError;

mod exports;
use runtime_tree::exports::ExportObj;
pub use runtime_tree::exports::ExportObject;

mod globals;
use runtime_tree::globals::Global;

mod language_types;
pub use runtime_tree::language_types::ExternalKindInstance;
pub use runtime_tree::language_types::ValueTypeProvider;

mod memory;
use runtime_tree::memory::Memory;

pub struct RuntimeModule {
    types: Vec<TypeDefinition>,
    functions: Vec<Box<Fn(&RuntimeModule, Vec<ValueTypeProvider>)->Vec<ValueTypeProvider>>>,
    tables: Vec<Table>,
    memories: Vec<Memory>,
    globals: Vec<Global>,
    exports: HashMap<String, ExternalKindInstance>,
    start_function: Option<usize>,
}

pub trait RuntimeModuleBuilder {
    fn build(&self, imports: HashMap<String, HashMap<String, ExternalKindInstance>>) -> Result<RuntimeModule, ParseError>;
}

impl RuntimeModuleBuilder for ParseModule {
    fn build(&self, imports: HashMap<String, HashMap<String, ExternalKindInstance>>) -> Result<RuntimeModule, ParseError> {
        let mut m = RuntimeModule {
            types: self.types.clone(),
            functions: vec![],
            tables: vec![],
            memories: vec![],
            globals: vec![],
            exports: HashMap::new(),
            start_function: None
        };
        m.build_imports(&self, imports);
        m.build_functions(&self);
        m.build_exports(&self);
        m.build_tables(&self);
        Ok(m)
    }
}

impl RuntimeModule {
    pub fn exports<'m>(&'m self) -> Box<ExportObject + 'm> {
        Box::new(ExportObj{module: &self})
    }

    fn build_imports(&mut self, parse_module: &ParseModule, mut imports: HashMap<String, HashMap<String, ExternalKindInstance>>) {
        for (namespace, values) in &(parse_module.imports) {
            for (name, value) in values {
                match *value {
                    ExternalKind::Function(i) => {
                        if let ExternalKindInstance::Function(x) = imports.get_mut(namespace).unwrap().remove(name).unwrap() {
                            self.functions.insert(i, x);
                        } else {
                            panic!("wrong type of import provided");
                        }
                    },
                    _ => panic!("not impl")
                }
            }
        }
    }

    fn build_tables(&mut self, parse_module: &ParseModule) {
        for parse_table in &(parse_module.tables) {
            self.tables.push((*parse_table).clone());
        }
    }

    fn build_functions(&mut self, parse_module: &ParseModule) {
        for f in parse_module.function_signatures.iter().zip(&parse_module.function_bodies) {
            let &TypeDefinition::Func(ref signature) = &parse_module.types[*f.0];
            let body = f.1;
            let args_size = signature.parameters.len();
            let locals_size = body.locals.len();
            let local_space_size = args_size + locals_size;

            let mut locals = Vec::with_capacity(local_space_size);
            locals.append(&mut signature.parameters.clone());
            locals.append(&mut body.locals.clone());

            let operations = body.code.clone();

            self.functions.push(Box::new(move |module, args|{
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
                            let &TypeDefinition::Func(ref signature) = &(module.types)[idx];
                            let mut args = vec![];
                            for param in &(signature.parameters) {
                                match *param {
                                    ValueType::I32 => {
                                        if let Some(ValueTypeProvider::I32(v)) = stack.pop() {
                                            args.push(ValueTypeProvider::I32(v));
                                        } else {
                                            panic!("wrong argument type");
                                        }
                                    },
                                    ValueType::I64 => {
                                        if let Some(ValueTypeProvider::I64(v)) = stack.pop() {
                                            args.push(ValueTypeProvider::I64(v));
                                        } else {
                                            panic!("wrong argument type");
                                        }
                                    },
                                    ValueType::F32 => {
                                        if let Some(ValueTypeProvider::F32(v)) = stack.pop() {
                                            args.push(ValueTypeProvider::F32(v));
                                        } else {
                                            panic!("wrong argument type");
                                        }
                                    },
                                    ValueType::F64 => {
                                        if let Some(ValueTypeProvider::F64(v)) = stack.pop() {
                                            args.push(ValueTypeProvider::F64(v));
                                        } else {
                                            panic!("wrong argument type");
                                        }
                                    },
                                }
                            }
                            if let Some(ValueTypeProvider::I32(index)) = stack.pop() {

                                println!("{:?}", index);

                                let &Table::AnyFunc{ref limits, ref values} = &(module.tables)[0];
                                let fn_index = values.get(index as usize).unwrap();
                                let callable = module.functions.get(*fn_index).unwrap();
                                for vtp in callable(module, args) {
                                    stack.push(vtp);
                                }
                            } else {
                                panic!("function not found or not indexed by i32");
                            }
                        },
                        Operation::I32Const(value) => {
                            stack.push(ValueTypeProvider::I32(value));
                        },
                        Operation::End => {
                            break
                        },
                        _ => panic!("not supported yet")
                    }
                    println!("stack after: {:?}", stack);
                }
                println!("returning: {:#?}", stack);
                vec![stack.pop().unwrap()]
            }));
        }
    }

    fn build_exports(&mut self, parse_module: &ParseModule) {
        for (key, value) in parse_module.exports.iter() {
            match *value {
                ExternalKind::Function(i) => {
                    self.exports.insert(key.clone(), ExternalKindInstance::Function(Box::new(move |module, args| {
                        return module.functions[i](module, args);
                    })))
                },
                _ => panic!("not suppeasdf")
            };
        }
    }
}
