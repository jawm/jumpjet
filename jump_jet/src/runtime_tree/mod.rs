use std;
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
            start_function: self.start_function
        };
        m.build_imports(&self, imports);
        m.build_functions(&self);
        m.build_exports(&self);
        m.build_tables(&self);

        if let Some(index) = m.start_function {
            let start_fn = m.functions.get(index).unwrap();
            start_fn(&m, vec![]);
        }
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

            let rets = signature.returns.clone();

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
                macro_rules! op {
                    ($op:expr, $($vt:pat)=>*) => {
                        op!(@i $op, stack, $($vt),*);
                    };
                    
                    (@i $op:expr, $stack:expr, $head:pat, $($vt:pat),*) => {
                        if let Some($head) = $stack.pop() {
                            op!(@i $op, $stack, $($vt),*);
                        } else {
                            println!("wrong ValueTypeProvider");
                        }
                    };
                    
                    (@i $op:expr, $stack:expr, $head:pat) => {
                        if let Some($head) = $stack.pop() {
                            $op;
                        } else {
                            println!("wrong ValueTypeProvider");
                        }
                    };

                    (@1i32, $a:ident, $op:expr) => {
                        op!(@i $op, stack, ValueTypeProvider::I32($a));
                    };
                    (@1i64, $a:ident, $op:expr) => {
                        op!(@i $op, stack, ValueTypeProvider::I64($a));
                    };
                    (@1f32, $a:ident, $op:expr) => {
                        op!(@i $op, stack, ValueTypeProvider::F32($a));
                    };
                    (@1f64 $a:ident, $op:expr) => {
                        op!(@i $op, stack, ValueTypeProvider::F64($a));
                    };
                }

                macro_rules! op_cmp {
                    (@i32, $a:ident, $b:ident, $cmp:expr) => {
                        op!(stack.push(ValueTypeProvider::I32($cmp as i32)), ValueTypeProvider::I32($a) => ValueTypeProvider::I32($b));
                    };
                    (@i64, $a:ident, $b:ident, $cmp:expr) => {
                        op!(stack.push(ValueTypeProvider::I32($cmp as i32)), ValueTypeProvider::I64($a) => ValueTypeProvider::I64($b));
                    };
                    (@f32, $a:ident, $b:ident, $cmp:expr) => {
                        op!(stack.push(ValueTypeProvider::I32($cmp as i32)), ValueTypeProvider::F32($a) => ValueTypeProvider::F32($b));
                    };
                    (@f64, $a:ident, $b:ident, $cmp:expr) => {
                        op!(stack.push(ValueTypeProvider::I32($cmp as i32)), ValueTypeProvider::F64($a) => ValueTypeProvider::F64($b));
                    };
                }

                for operation in &(operations) {
                    match *operation {
                        Operation::Unreachable => panic!("Unreachable code executed"),
                        Operation::Nop => {},
                        Operation::Block(ref b) => {
                            // TODO maybe do stuff recursively here, because I'm lazy af
                        },
                        Operation::Loop(ref b) => {

                        },
                        Operation::If(ref b) => {

                        },
                        Operation::Else => {

                        },
                        Operation::End => {
                            break
                        },
                        Operation::Branch(i) => {

                        },
                        Operation::BranchIf(i) => {

                        },
                        Operation::BranchTable(ref bt) => {

                        },
                        Operation::Return => {

                        },
                        Operation::Call(i) => {

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
                                let &Table::AnyFunc{ref limits, ref values} = &(module.tables)[0];
                                let fn_index = values.get(index as usize).unwrap();
                                let callable = module.functions.get(*fn_index).unwrap();
                                for ValueTypeProvider in callable(module, args) {
                                    stack.push(ValueTypeProvider);
                                }
                            } else {
                                panic!("function not found or not indexed by i32");
                            }
                        },
                        Operation::Drop => {stack.pop();},
                        Operation::Select => {
                            let a = stack.pop().unwrap();
                            let b = stack.pop().unwrap();
                            if std::mem::discriminant(&a) != std::mem::discriminant(&b) {
                                panic!("ValueTypeProvider must be the same for 'a' and 'b' in 'select' op!")
                            }
                            op!(if v != 0 {
                                stack.push(a);
                            } else {
                                stack.push(b);
                            }, ValueTypeProvider::I32(v));
                        },
                        Operation::GetLocal(idx) => stack.push(local_space[idx].clone()),
                        Operation::SetLocal(idx) => {

                        },
                        Operation::TeeLocal(idx) => {

                        },
                        Operation::GetGlobal(idx) => {
                        },
                        Operation::SetGlobal(idx) => {

                        },
                        Operation::I32Load(ref mem) => {},
                        Operation::I64Load(ref mem) => {},
                        Operation::F32Load(ref mem) => {},
                        Operation::F64Load(ref mem) => {},
                        Operation::I32Load8S(ref mem) => {},
                        Operation::I32Load8U(ref mem) => {},
                        Operation::I32Load16S(ref mem) => {},
                        Operation::I32Load16U(ref mem) => {},
                        Operation::I64Load8S(ref mem) => {},
                        Operation::I64Load8U(ref mem) => {},
                        Operation::I64Load16S(ref mem) => {},
                        Operation::I64Load16U(ref mem) => {},
                        Operation::I64Load32S(ref mem) => {},
                        Operation::I64Load32U(ref mem) => {},
                        Operation::I32Store(ref mem) => {},
                        Operation::I64Store(ref mem) => {},
                        Operation::F32Store(ref mem) => {},
                        Operation::F64Store(ref mem) => {},
                        Operation::I32Store8(ref mem) => {},
                        Operation::I32Store16(ref mem) => {},
                        Operation::I64Store8(ref mem) => {},
                        Operation::I64Store16(ref mem) => {},
                        Operation::I64Store32(ref mem) => {},
                        Operation::CurrentMemory(_) => {},
                        Operation::GrowMemory(_) => {},
                        Operation::I32Const(value) => {
                            stack.push(ValueTypeProvider::I32(value));
                        },
                        Operation::I64Const(value) => {
                            stack.push(ValueTypeProvider::I64(value));
                        },
                        Operation::F32Const(value) => {
                            stack.push(ValueTypeProvider::F32(value));
                        },
                        Operation::F64Const(value) => {
                            stack.push(ValueTypeProvider::F64(value));
                        },
                        Operation::I32Eqz => {
                            op!(@1i32, a, stack.push(ValueTypeProvider::I32((a == 0) as i32)));
                        },
                        Operation::I32Eq => {op_cmp!(@i32, a, b, a==b);},
                        Operation::I32Ne => {op_cmp!(@i32, a, b, a!=b);},
                        Operation::I32LtS => {op_cmp!(@i32, a, b, a<b);},
                        Operation::I32LtU => {op_cmp!(@i32, a, b, (a as u32) < (b as u32));},
                        Operation::I32GtS => {op_cmp!(@i32, a, b, a>b);},
                        Operation::I32GtU => {op_cmp!(@i32, a, b, (a as u32) > (b as u32));},
                        Operation::I32LeS => {op_cmp!(@i32, a, b, a<=b);},
                        Operation::I32LeU => {op_cmp!(@i32, a, b, (a as u32) <= (b as u32));},
                        Operation::I32GeS => {op_cmp!(@i32, a, b, a>=b);},
                        Operation::I32GeU => {op_cmp!(@i32, a, b, (a as u32) >= (b as u32));},
                        Operation::I64Eqz => {
                            op!(@1i64, a, stack.push(ValueTypeProvider::I32((a == 0) as i32)));
                        },
                        Operation::I64Eq => {op_cmp!(@i64, a, b, a==b);},
                        Operation::I64Ne => {op_cmp!(@i64, a, b, a!=b);},
                        Operation::I64LtS => {op_cmp!(@i64, a, b, a<b);},
                        Operation::I64LtU => {op_cmp!(@i64, a, b, (a as u32) < (b as u32));},
                        Operation::I64GtS => {op_cmp!(@i64, a, b, a>b);},
                        Operation::I64GtU => {op_cmp!(@i64, a, b, (a as u32) > (b as u32));},
                        Operation::I64LeS => {op_cmp!(@i64, a, b, a<=b);},
                        Operation::I64LeU => {op_cmp!(@i64, a, b, (a as u32) <= (b as u32));},
                        Operation::I64GeS => {op_cmp!(@i64, a, b, a>=b);},
                        Operation::I64GeU => {op_cmp!(@i64, a, b, (a as u32) >= (b as u32));},
                        Operation::F32Eq => {op_cmp!(@f32, a, b, a==b);},
                        Operation::F32Ne => {op_cmp!(@f32, a, b, a!=b);},
                        Operation::F32Lt => {op_cmp!(@f32, a, b, a<b);},
                        Operation::F32Gt => {op_cmp!(@f32, a, b, a>b);},
                        Operation::F32Le => {op_cmp!(@f32, a, b, a<=b);},
                        Operation::F32Ge => {op_cmp!(@f32, a, b, a>=b);},
                        Operation::F64Eq => {op_cmp!(@f64, a, b, a==b);},
                        Operation::F64Ne => {op_cmp!(@f64, a, b, a!=b);},
                        Operation::F64Lt => {op_cmp!(@f64, a, b, a<b);},
                        Operation::F64Gt => {op_cmp!(@f64, a, b, a>b);},
                        Operation::F64Le => {op_cmp!(@f64, a, b, a<=b);},
                        Operation::F64Ge => {op_cmp!(@f64, a, b, a>=b);},
                        Operation::I32Clz => {
                            op!(@1i32, a, stack.push(ValueTypeProvider::I32(a.leading_zeros() as i32)));
                        },
                        Operation::I32Ctz => {
                            op!(@1i32, a, stack.push(ValueTypeProvider::I32(a.trailing_zeros() as i32)));
                        },
                        Operation::I32Popcnt => {
                            op!(@1i32, a, stack.push(ValueTypeProvider::I32(a.count_ones() as i32)));
                        },
                        Operation::I32Add => {
                            op!(stack.push(ValueTypeProvider::I32(a + b)),
                                ValueTypeProvider::I32(a) => ValueTypeProvider::I32(b));
                        }
                        Operation::I32Sub => {
                            op!(stack.push(ValueTypeProvider::I32(a - b)),
                                ValueTypeProvider::I32(a) => ValueTypeProvider::I32(b));
                        },
                        Operation::I32Mul => {
                            op!(stack.push(ValueTypeProvider::I32(a * b)),
                                ValueTypeProvider::I32(a) => ValueTypeProvider::I32(b));
                        },
                        Operation::I32DivS => {},
                        Operation::I32DivU => {},
                        Operation::I32RemS => {},
                        Operation::I32RemU => {},
                        Operation::I32And => {
                            op!(stack.push(ValueTypeProvider::I32(a & b)),
                                ValueTypeProvider::I32(a) => ValueTypeProvider::I32(b));
                        },
                        Operation::I32Or => {
                            op!(stack.push(ValueTypeProvider::I32(a | b)),
                                ValueTypeProvider::I32(a) => ValueTypeProvider::I32(b));
                        },
                        Operation::I32Xor => {
                            op!(stack.push(ValueTypeProvider::I32(a ^ b)),
                                ValueTypeProvider::I32(a) => ValueTypeProvider::I32(b));
                        },
                        Operation::I32Shl => {},
                        Operation::I32ShrS => {},
                        Operation::I32ShrU => {},
                        Operation::I32Rotl => {},
                        Operation::I32Rotr => {},
                        Operation::I64Clz => {
                            op!(stack.push(ValueTypeProvider::I32(a.leading_zeros() as i32)), 
                                ValueTypeProvider::I64(a));
                        },
                        Operation::I64Ctz => {
                            op!(stack.push(ValueTypeProvider::I32(a.trailing_zeros() as i32)), 
                                ValueTypeProvider::I64(a));
                        },
                        Operation::I64Popcnt => {
                            op!(stack.push(ValueTypeProvider::I32(a.count_ones() as i32)), 
                                ValueTypeProvider::I64(a));
                        },
                        Operation::I64Add => {},
                        Operation::I64Sub => {},
                        Operation::I64Mul => {},
                        Operation::I64DivS => {},
                        Operation::I64DivU => {},
                        Operation::I64RemS => {},
                        Operation::I64RemU => {},
                        Operation::I64And => {},
                        Operation::I64Or => {},
                        Operation::I64Xor => {},
                        Operation::I64Shl => {},
                        Operation::I64ShrS => {},
                        Operation::I64ShrU => {},
                        Operation::I64Rotl => {},
                        Operation::I64Rotr => {},
                        Operation::F32Abs => {},
                        Operation::F32Neg => {},
                        Operation::F32Ceil => {},
                        Operation::F32Floor => {},
                        Operation::F32Trunc => {},
                        Operation::F32Nearest => {},
                        Operation::F32Sqrt => {},
                        Operation::F32Add => {},
                        Operation::F32Sub => {},
                        Operation::F32Mul => {},
                        Operation::F32Div => {},
                        Operation::F32Min => {},
                        Operation::F32Max => {},
                        Operation::F32Copysign => {},
                        Operation::F64Abs => {},
                        Operation::F64Neg => {},
                        Operation::F64Ceil => {},
                        Operation::F64Floor => {},
                        Operation::F64Trunc => {},
                        Operation::F64Nearest => {},
                        Operation::F64Sqrt => {},
                        Operation::F64Add => {},
                        Operation::F64Sub => {},
                        Operation::F64Mul => {},
                        Operation::F64Div => {},
                        Operation::F64Min => {},
                        Operation::F64Max => {},
                        Operation::F64Copysign => {},
                        Operation::I32WrapI64 => {},
                        Operation::I32TruncSF32 => {},
                        Operation::I32TruncUF32 => {},
                        Operation::I32TruncSF64 => {},
                        Operation::I32TruncUF64 => {},
                        Operation::I64ExtendSI32 => {},
                        Operation::I64ExtendUI32 => {},
                        Operation::I64TruncSF32 => {},
                        Operation::I64TruncUF32 => {},
                        Operation::I64TruncSF64 => {},
                        Operation::I64TruncUF64 => {},
                        Operation::F32ConvertSI32 => {},
                        Operation::F32ConvertUI32 => {},
                        Operation::F32ConvertSI64 => {},
                        Operation::F32ConvertUI64 => {},
                        Operation::F32DemoteF64 => {},
                        Operation::F64ConvertSI32 => {},
                        Operation::F64ConvertUI32 => {},
                        Operation::F64ConvertSI64 => {},
                        Operation::F64ConvertUI64 => {},
                        Operation::F64PromoteF32 => {},
                        Operation::I32ReinterpretF32 => {},
                        Operation::I64ReinterpretF64 => {},
                        Operation::F32ReinterpretI32 => {},
                        Operation::F64ReinterpretI64 => {}
                    }
                }
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
            }));
        }
    }

    fn build_exports(&mut self, parse_module: &ParseModule) {
        for (key, value) in parse_module.exports.iter() {
            match *value {
                ExternalKind::Function(i) => {
                    self.exports.insert(key.clone(), ExternalKindInstance::Function(Box::new(move |module, args| {
                        return module.functions[i](module, args);
                    })));
                },
                ExternalKind::Memory(i) => {
                    //todo figure out how to properly export memories... this was just a hack to test more programs.
                    self.exports.insert(key.clone(), ExternalKindInstance::Memory(i));
                },
                _ => panic!("not suppeasdf")
            };
        }
    }
}
