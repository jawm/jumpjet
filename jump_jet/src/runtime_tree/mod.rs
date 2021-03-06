extern crate byteorder;
use self::byteorder::LittleEndian;
use self::byteorder::ReadBytesExt;
use self::byteorder::WriteBytesExt;

use std;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;

use parse_tree::functions::FuncSignature;
use parse_tree::language_types::Block;
use parse_tree::language_types::BlockType;
use parse_tree::language_types::ExternalKind;
use parse_tree::language_types::Operation;
use parse_tree::language_types::ValueType;
use parse_tree::memory::Memory;
use parse_tree::ParseModule;
use parse_tree::tables::Table;
use parse_tree::types::TypeDefinition;

use parser::ParseError;

mod exports;
use runtime_tree::exports::ExportObj;
pub use runtime_tree::exports::ExportObject;

mod globals;
use runtime_tree::globals::Global;
use runtime_tree::language_types::StackFrame;

mod language_types;
pub use runtime_tree::language_types::Import;
use runtime_tree::language_types::Execute;
pub use runtime_tree::language_types::ExternalKindInstance;
pub use runtime_tree::language_types::ValueTypeProvider;

//pub type Func = Box<Fn(&mut ModuleInstanceData, Vec<ValueTypeProvider>)->Vec<ValueTypeProvider>>;
pub struct Func {
    signature: FuncSignature,
    callable: Box<Fn(&mut ModuleInstanceData, Vec<ValueTypeProvider>)->Vec<ValueTypeProvider>>
}

pub struct ModuleTemplate {
    exports: HashMap<String, ExternalKind>,
    start_function: Option<usize>,
    types: Vec<TypeDefinition>,
    globals: Vec<ValueTypeProvider>,
    memories: Vec<Memory>,
    functions: Vec<Func>,
    tables: Vec<Table>
}

impl ModuleTemplate {
    // TODO not ParseError
    pub fn instantiate(&self) -> Result<ModuleInstance, ParseError> {
        Ok(ModuleInstance {
            types: self.types.clone(),
            exports: self.build_exports(),
            globals: RefCell::new(vec![]),
            memories: RefCell::new(vec![]),
            functions: &self.functions,
            tables: RefCell::new(vec![]),
        })
    }

    fn build_exports(&self) -> HashMap<String, ExternalKindInstance> {
        let mut exports = HashMap::new();
        for (key, value) in self.exports.iter() {
            exports.insert(key.clone(), match *value {
                ExternalKind::Function(f) => ExternalKindInstance::Function(
                    Func{
                        signature: self.functions[f].signature.clone(),
                        callable: Box::new(move |module, args|{
                            println!("getting function {:?}/{:?}", f, module.functions.len());
                            (module.functions[f].callable)(module, args)
                    })}
                ),
                _ => ExternalKindInstance::Memory(0)
            });
        }
        exports
    }

}

pub struct ModuleInstance<'a> {
    types: Vec<TypeDefinition>,
    exports: HashMap<String, ExternalKindInstance>,
    globals: RefCell<Vec<ValueTypeProvider>>,
    memories: RefCell<Vec<Memory>>,
    functions: &'a Vec<Func>, // TODO we might not need this?
    tables: RefCell<Vec<Table>>
}

impl<'a> ModuleInstance<'a> {
    pub fn exports(&'a mut self) -> Box<ExportObject + 'a> {
        Box::new(ExportObj {
            module: self
        })
    }

    pub fn get_frame(&self) -> ModuleInstanceData {
        ModuleInstanceData {
            types: self.types.clone(),
            globals: self.globals.borrow_mut(),
            functions: self.functions,
            memories: self.memories.borrow_mut(),
            tables: self.tables.borrow_mut()
        }
    }
}

pub struct ModuleInstanceData<'a> {
    types: Vec<TypeDefinition>,
    globals: RefMut<'a, Vec<ValueTypeProvider>>,
    memories: RefMut<'a, Vec<Memory>>,
    functions: &'a Vec<Func>,
    tables: RefMut<'a, Vec<Table>>
}

pub trait ModuleTemplateBuilder {
    // TODO don't use ParseError
    fn build(&self, imports: HashMap<String, HashMap<String, Import>>) -> Result<ModuleTemplate, ParseError>;
}

impl ModuleTemplateBuilder for ParseModule {
    fn build(&self, mut imports: HashMap<String, HashMap<String, Import>>) -> Result<ModuleTemplate, ParseError> {
        println!("{:?}", self.exports);
        Ok(ModuleTemplate {
            exports: self.exports.clone(),
            functions: self.build_functions(&mut imports),
            globals: vec![],
            memories: self.memories.clone(),
            start_function: None,
            tables: vec![],
            types: self.types.clone()
        })
    }
}

impl ParseModule {
    pub fn build_functions(&self, imports: &mut HashMap<String, HashMap<String, Import>>) -> Vec<Func> {
        let mut functions: Vec<Func> = vec![];
        for imported_module in &(self.imports) {
            for imported_item in imported_module.1.iter() {
                if let ExternalKind::Function(signature_idx) = *imported_item.1 {
                    if let Some(mut map) = imports.remove(imported_module.0) {
                        if let Some(Import::Function(f)) = map.remove(imported_item.0) {
                            match self.types[signature_idx].clone() {
                                TypeDefinition::Func(signature) => {
                                    functions.push(Func {
                                        signature,
                                        callable: f
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        for func in self.function_signatures.iter().zip(self.function_bodies.iter()) {

            let &TypeDefinition::Func(ref signature) = &self.types[*func.0];
            let body = func.1;
            let args_size = signature.parameters.len();
            let locals_size = body.locals.len();
            let local_space_size = args_size + locals_size;

            let mut locals = Vec::with_capacity(local_space_size);
            locals.append(&mut signature.parameters.clone());
            locals.append(&mut body.locals.clone());

            let operations = body.code.clone();

            let rets = signature.returns.clone();

            let block_type = if rets.len() == 0 {
                BlockType::Empty
            } else {
                BlockType::Value(rets.get(0).unwrap().clone())
            };
            let block = Block {
                operations: operations.clone(),
                block_type
            };
            functions.push(Func{
                signature: signature.clone(),
                callable: Box::new(move |mut module, args| {
                println!("ayo in the function");

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
                let mut control_flow_stack: Vec<(usize, BlockType)> = vec![];

                block.execute(&mut StackFrame {
                    data: &mut module,
                    locals: &mut local_space,
                    stack: &mut stack
                });

                let mut results = vec![];
                for ret in &rets {
                    match *ret {
                        ValueType::I32 => {
                            if let Some(ValueTypeProvider::I32(i)) = stack.pop() {
                                results.push(ValueTypeProvider::I32(i));
                            }
                        },
                        ValueType::I64 => {
                            if let Some(ValueTypeProvider::I64(i)) = stack.pop() {
                                results.push(ValueTypeProvider::I64(i));
                            }
                        },
                        ValueType::F32 => {
                            if let Some(ValueTypeProvider::F32(i)) = stack.pop() {
                                results.push(ValueTypeProvider::F32(i));
                            }
                        },
                        ValueType::F64 => {
                            if let Some(ValueTypeProvider::F64(i)) = stack.pop() {
                                results.push(ValueTypeProvider::F64(i));
                            }
                        },
                        _ => {}
                    }
                }
                return results;
            })});
        }
        functions
    }
}
