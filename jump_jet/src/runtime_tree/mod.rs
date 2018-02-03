extern crate byteorder;
use self::byteorder::LittleEndian;
use self::byteorder::ReadBytesExt;
use self::byteorder::WriteBytesExt;

use std;
use std::collections::HashMap;

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

mod language_types;
pub use runtime_tree::language_types::ExternalKindInstance;
pub use runtime_tree::language_types::ValueTypeProvider;

pub type Func = Box<Fn(&mut CallFrame, Vec<ValueTypeProvider>)->Vec<ValueTypeProvider>>;

pub struct RuntimeModule {
    types: Vec<TypeDefinition>,
    functions: Vec<Func>,
    tables: Vec<Table>,
    memories: Vec<Memory>,
    globals: Vec<ValueTypeProvider>,
    exports: HashMap<String, ExternalKindInstance>,
    start_function: Option<usize>,
}

pub struct CallFrame<'a> {
    types: &'a Vec<TypeDefinition>,
    globals: &'a mut Vec<ValueTypeProvider>,
    memories: &'a mut Vec<Memory>,
    functions: &'a Vec<Func>,
    tables: &'a Vec<Table>
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
        m.build_memories(&self);
        m.build_functions(&self);
        m.build_exports(&self);
        m.build_tables(&self);

        if let Some(index) = m.start_function {
            let start_fn = m.functions.get(index).unwrap();
            let mut call_frame = CallFrame {
                types: & m.types,
                tables: & m.tables,
                memories: &mut m.memories,
                globals: &mut m.globals,
                functions: & m.functions
            };
            start_fn(&mut call_frame, vec![]);
        }
        Ok(m)
    }
}

impl RuntimeModule {
    pub fn exports<'m>(&'m mut self) -> Box<ExportObject + 'm> {
        Box::new(ExportObj{module: self})
    }

    fn build_memories(&mut self, parse_module: &ParseModule) {
        for m in &(parse_module.memories) {
            self.memories.push((*m).clone());
        }
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
                    ($($a:ident:$b:ident),* | $r:ident => $op:expr) => {
                        op!(@a stack.push(ValueTypeProvider::$r($op)), $($a:$b),*);
                    };

                    ($($a:ident:$b:ident),* | @bool => $op:expr) => {
                        op!(@a stack.push(ValueTypeProvider::I32($op as i32)), $($a:$b),*);
                    };

                    ($($a:ident:$b:ident),* | @any => $op:expr) => {
                        op!(@a stack.push($op), $($a:$b),*);
                    };

                    (@a $op:expr, $hn:ident:$ht:ident, $($a:ident:$b:ident),*) => {
                        if let Some(ValueTypeProvider::$ht($hn)) = stack.pop() {
                            op!(@a $op, $($a:$b),*);
                        }
                    };

                    (@a $op:expr, $hn:ident:$ht:ident) => {
                        if let Some(ValueTypeProvider::$ht($hn)) = stack.pop() {
                            $op;
                        }
                    };
                }

                macro_rules! mem_op {
                    // Read a 32bit int onto the stack
                    // mem_op!(mem => I32(i32));

                    // Write a 32bit int onto the stack
                    // mem_op!(I32(i32) => mem);

                    ($a:expr => $b:ident($c:ty,$d:ty)) => {
                        let offset = ($a.flags + $a.offset) as usize;
                        let size = std::mem::size_of::<$c>() as usize;
                        let mut a = &module.memories[0].values[offset..offset+8];
                        let value = a.read_int::<LittleEndian>(size).unwrap() as $d;
                        stack.push(ValueTypeProvider::$b(value));
                    };

                    ($a:expr => $b:ident($c:ty)) => {
                        mem_op!($a => $b($c,$c));
                    };

                    ($a:ident($c:ty) => $d:expr) => {
                        if let Some(ValueTypeProvider::$a(value)) = stack.pop() {
                            let offset = ($d.flags + $d.offset) as usize;
                            let size = std::mem::size_of::<$c>() as usize;
                            let mut a = &mut module.memories[0].values[offset..offset+8];
                            a.write_int::<LittleEndian>(value as $c as i64, size);
                        } else {
                            panic!("VTP was wrong type or not present!");
                        }
                    };
                }

                for operation in &(operations) {
                    match *operation {
                        Operation::Unreachable => panic!("Unreachable code executed"),
                        Operation::Nop => {},
                        Operation::Block(ref b) => {
                            // TODO maybe do stuff recursively here, because I'm lazy af
                        },
                        Operation::Loop(ref b) => {},
                        Operation::If(ref b) => {},
                        Operation::Else => {},
                        Operation::End => {break},
                        Operation::Branch(i) => {},
                        Operation::BranchIf(i) => {},
                        Operation::BranchTable(ref bt) => {},
                        Operation::Return => {},
                        Operation::Call(i) => {},
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
                            op!(v:I32 | @any => {
                                if v != 0 {
                                    a
                                } else {
                                    b
                                }
                            });
                        },
                        Operation::GetLocal(idx) => stack.push(local_space[idx].clone()),
                        Operation::SetLocal(idx) => {
                            if let Some(vtp) = stack.pop() {
                                if std::mem::discriminant(&local_space[idx]) == std::mem::discriminant(&vtp) {
                                    local_space[idx] = vtp;
                                } else {
                                    panic!("Wrong type provided for local set");
                                }
                            } else {
                                panic!("no values on stack");
                            }
                        },
                        Operation::TeeLocal(idx) => {
                            if let Some(vtp) = stack.pop() {
                                if std::mem::discriminant(&local_space[idx]) == std::mem::discriminant(&vtp) {
                                    local_space[idx] = vtp.clone();
                                    stack.push(vtp);
                                } else {
                                    panic!("Wrong type provided for local set");
                                }
                            } else {
                                panic!("no values on stack");
                            }
                        },
                        Operation::GetGlobal(idx) => stack.push(module.globals[idx].clone()),
                        Operation::SetGlobal(idx) => {
                            if let Some(vtp) = stack.pop() {
                                if std::mem::discriminant(&module.globals[idx]) == std::mem::discriminant(&vtp) {
                                    module.globals[idx] = vtp;
                                } else {
                                    panic!("Wrong type provided for local set");
                                }
                            } else {
                                panic!("no values on stack");
                            }
                        },
                        Operation::I32Load(ref mem) => {mem_op!(mem => I32(i32));},
                        Operation::I64Load(ref mem) => {mem_op!(mem => I64(i64));},
                        Operation::F32Load(ref mem) => {mem_op!(mem => F32(f32));},
                        Operation::F64Load(ref mem) => {mem_op!(mem => F64(f64));},
                        Operation::I32Load8S(ref mem) => {mem_op!(mem => I32(i8,i32));},
                        Operation::I32Load8U(ref mem) => {mem_op!(mem => I32(u8,i32));},
                        Operation::I32Load16S(ref mem) => {mem_op!(mem => I32(i16,i32));},
                        Operation::I32Load16U(ref mem) => {mem_op!(mem => I32(u16,i32));},
                        Operation::I64Load8S(ref mem) => {mem_op!(mem => I64(i8,i64));},
                        Operation::I64Load8U(ref mem) => {mem_op!(mem => I64(u8,i64));},
                        Operation::I64Load16S(ref mem) => {mem_op!(mem => I64(i16,i64));},
                        Operation::I64Load16U(ref mem) => {mem_op!(mem => I64(u16,i64));},
                        Operation::I64Load32S(ref mem) => {mem_op!(mem => I64(i32,i64));},
                        Operation::I64Load32U(ref mem) => {mem_op!(mem => I64(u32,i64));},
                        Operation::I32Store(ref mem) => {mem_op!(I32(i32) => mem);},
                        Operation::I64Store(ref mem) => {mem_op!(I64(i64) => mem);},
                        Operation::F32Store(ref mem) => {mem_op!(F32(f32) => mem);},
                        Operation::F64Store(ref mem) => {mem_op!(F64(f64) => mem);},
                        Operation::I32Store8(ref mem) => {mem_op!(I32(i8) => mem);},
                        Operation::I32Store16(ref mem) => {mem_op!(I32(i16) => mem);},
                        Operation::I64Store8(ref mem) => {mem_op!(I64(i8) => mem);},
                        Operation::I64Store16(ref mem) => {mem_op!(I64(i16) => mem);},
                        Operation::I64Store32(ref mem) => {mem_op!(I64(i32) => mem);},
                        Operation::CurrentMemory(_) => {},
                        Operation::GrowMemory(_) => {},
                        Operation::I32Const(value) => {stack.push(ValueTypeProvider::I32(value))},
                        Operation::I64Const(value) => {stack.push(ValueTypeProvider::I64(value))},
                        Operation::F32Const(value) => {stack.push(ValueTypeProvider::F32(value))},
                        Operation::F64Const(value) => {stack.push(ValueTypeProvider::F64(value))},
                        Operation::I32Eqz => {op!(a:I32 | @bool => a==0)},
                        Operation::I32Eq => {op!(a:I32,b:I32 | @bool => a==b)},
                        Operation::I32Ne => {op!(a:I32, b:I32 | @bool => a!=b)},
                        Operation::I32LtS => {op!(a:I32, b:I32 | @bool => a<b)},
                        Operation::I32LtU => {op!(a:I32, b:I32 | @bool => (a as u32) < (b as u32))},
                        Operation::I32GtS => {op!(a:I32, b:I32 | @bool => a>b)},
                        Operation::I32GtU => {op!(a:I32, b:I32 | @bool => (a as u32) > (b as u32))},
                        Operation::I32LeS => {op!(a:I32, b:I32 | @bool => a<=b)},
                        Operation::I32LeU => {op!(a:I32, b:I32 | @bool => (a as u32) <= (b as u32))},
                        Operation::I32GeS => {op!(a:I32, b:I32 | @bool => a>=b)},
                        Operation::I32GeU => {op!(a:I32, b:I32 | @bool => (a as u32) >= (b as u32))},
                        Operation::I64Eqz => {op!(a:I32 | @bool => a==0)},
                        Operation::I64Eq => {op!(a:I64, b:I64 | @bool => a==b)},
                        Operation::I64Ne => {op!(a:I64, b:I64 | @bool => a!=b)},
                        Operation::I64LtS => {op!(a:I64, b:I64 | @bool => a<b)},
                        Operation::I64LtU => {op!(a:I64, b:I64 | @bool => (a as u32) < (b as u32))},
                        Operation::I64GtS => {op!(a:I64, b:I64 | @bool => a>b)},
                        Operation::I64GtU => {op!(a:I64, b:I64 | @bool => (a as u32) > (b as u32))},
                        Operation::I64LeS => {op!(a:I64, b:I64 | @bool => a<=b)},
                        Operation::I64LeU => {op!(a:I64, b:I64 | @bool => (a as u32) <= (b as u32))},
                        Operation::I64GeS => {op!(a:I64, b:I64 | @bool => a>=b)},
                        Operation::I64GeU => {op!(a:I64, b:I64 | @bool => (a as u32) >= (b as u32))},
                        Operation::F32Eq => {op!(a:F32, b:F32 | @bool => a==b)},
                        Operation::F32Ne => {op!(a:F32, b:F32 | @bool => a!=b)},
                        Operation::F32Lt => {op!(a:F32, b:F32 | @bool => a<b)},
                        Operation::F32Gt => {op!(a:F32, b:F32 | @bool => a>b)},
                        Operation::F32Le => {op!(a:F32, b:F32 | @bool => a<=b)},
                        Operation::F32Ge => {op!(a:F32, b:F32 | @bool => a>=b)},
                        Operation::F64Eq => {op!(a:F64, b:F64 | @bool => a==b)},
                        Operation::F64Ne => {op!(a:F64, b:F64 | @bool => a!=b)},
                        Operation::F64Lt => {op!(a:F64, b:F64 | @bool => a<b)},
                        Operation::F64Gt => {op!(a:F64, b:F64 | @bool => a>b)},
                        Operation::F64Le => {op!(a:F64, b:F64 | @bool => a<=b)},
                        Operation::F64Ge => {op!(a:F64, b:F64 | @bool => a>=b)},
                        Operation::I32Clz => {op!(a:I32 | I32 => a.leading_zeros() as i32)},
                        Operation::I32Ctz => {op!(a:I32 | I32 => a.trailing_zeros() as i32)},
                        Operation::I32Popcnt => {op!(a:I32 | I32 => a.count_ones() as i32)},
                        Operation::I32Add => {op!(a:I32, b:I32 | I32 => a+b)}
                        Operation::I32Sub => {op!(a:I32, b:I32 | I32 => a - b)},
                        Operation::I32Mul => {op!(a:I32, b:I32 | I32 => a * b)},
                        Operation::I32DivS => {op!(a:I32, b:I32 | I32 => a / b)},
                        Operation::I32DivU => {op!(a:I32, b:I32 | I32 => ((a as u32) / (b as u32)) as i32)},
                        Operation::I32RemS => {op!(a:I32, b:I32 | I32 => a % b)},
                        Operation::I32RemU => {op!(a:I32, b:I32 | I32 => ((a as u32) % (b as u32)) as i32)},
                        Operation::I32And => {op!(a:I32, b:I32 | I32 => a & b)},
                        Operation::I32Or => {op!(a:I32, b:I32 | I32 => a | b)},
                        Operation::I32Xor => {op!(a:I32, b:I32 | I32 => a ^ b)},
                        Operation::I32Shl => {op!(a:I32, b:I32 | I32 => a << b)},
                        Operation::I32ShrS => {op!(a:I32, b:I32 | I32 => a >> b)},
                        Operation::I32ShrU => {op!(a:I32, b:I32 | I32 => ((a as u32) >> (b as u32)) as i32)},
                        Operation::I32Rotl => {op!(a:I32, b:I32 | I32 => a.rotate_left(b as u32))},
                        Operation::I32Rotr => {op!(a:I32, b:I32 | I32 => a.rotate_right(b as u32))},
                        Operation::I64Clz => {op!(a:I64 | I32 => a.leading_zeros() as i32)},
                        Operation::I64Ctz => {op!(a:I64 | I32 => a.trailing_zeros() as i32)},
                        Operation::I64Popcnt => {op!(a:I64 | I32 => a.count_ones() as i32)},
                        Operation::I64Add => {op!(a:I64, b:I64 | I64 => a+b)}
                        Operation::I64Sub => {op!(a:I64, b:I64 | I64 => a - b)},
                        Operation::I64Mul => {op!(a:I64, b:I64 | I64 => a * b)},
                        Operation::I64DivS => {op!(a:I64, b:I64 | I64 => a / b)},
                        Operation::I64DivU => {op!(a:I64, b:I64 | I64 => ((a as u64) / (b as u64)) as i64)},
                        Operation::I64RemS => {op!(a:I64, b:I64 | I64 => a % b)},
                        Operation::I64RemU => {op!(a:I64, b:I64 | I64 => ((a as u64) % (b as u64)) as i64)},
                        Operation::I64And => {op!(a:I64, b:I64 | I64 => a & b)},
                        Operation::I64Or => {op!(a:I64, b:I64 | I64 => a | b)},
                        Operation::I64Xor => {op!(a:I64, b:I64 | I64 => a ^ b)},
                        Operation::I64Shl => {op!(a:I64, b:I64 | I64 => a << b)},
                        Operation::I64ShrS => {op!(a:I64, b:I64 | I64 => a >> b)},
                        Operation::I64ShrU => {op!(a:I64, b:I64 | I64 => ((a as u64) >> (b as u64)) as i64)},
                        Operation::I64Rotl => {op!(a:I64, b:I64 | I64 => a.rotate_left(b as u32))},
                        Operation::I64Rotr => {op!(a:I64, b:I64 | I64 => a.rotate_right(b as u32))},
                        Operation::F32Abs => {op!(a:F32 | F32 => a.abs())},
                        Operation::F32Neg => {op!(a:F32 | F32 => -a)},
                        Operation::F32Ceil => {op!(a:F32 | F32 => a.ceil())},
                        Operation::F32Floor => {op!(a:F32 | F32 => a.floor())},
                        Operation::F32Trunc => {op!(a:F32 | F32 => a.trunc())},
                        Operation::F32Nearest => {op!(a:F32 | F32 => a.round())},
                        Operation::F32Sqrt => {op!(a:F32 | F32 => a.sqrt())},
                        Operation::F32Add => {op!(a:F32, b:F32 | F32 => a+b)},
                        Operation::F32Sub => {op!(a:F32, b:F32 | F32 => a-b)},
                        Operation::F32Mul => {op!(a:F32, b:F32 | F32 => a*b)},
                        Operation::F32Div => {op!(a:F32, b:F32 | F32 => a/b)},
                        Operation::F32Min => {op!(a:F32, b:F32 | F32 => a.min(b))},
                        Operation::F32Max => {op!(a:F32, b:F32 | F32 => a.max(b))},
                        Operation::F32Copysign => {op!(a:F32, b:F32 | F32 => {
                            ((a as u32) | ((b as u32) & (1 << 31))) as f32
                        })},
                        Operation::F64Abs => {op!(a:F64 | F64 => a.abs())},
                        Operation::F64Neg => {op!(a:F64 | F64 => -a)},
                        Operation::F64Ceil => {op!(a:F64 | F64 => a.ceil())},
                        Operation::F64Floor => {op!(a:F64 | F64 => a.floor())},
                        Operation::F64Trunc => {op!(a:F64 | F64 => a.trunc())},
                        Operation::F64Nearest => {op!(a:F64 | F64 => a.round())},
                        Operation::F64Sqrt => {op!(a:F64 | F64 => a.sqrt())},
                        Operation::F64Add => {op!(a:F64, b:F64 | F64 => a+b)},
                        Operation::F64Sub => {op!(a:F64, b:F64 | F64 => a-b)},
                        Operation::F64Mul => {op!(a:F64, b:F64 | F64 => a*b)},
                        Operation::F64Div => {op!(a:F64, b:F64 | F64 => a/b)},
                        Operation::F64Min => {op!(a:F64, b:F64 | F64 => a.min(b))},
                        Operation::F64Max => {op!(a:F64, b:F64 | F64 => a.max(b))},
                        Operation::F64Copysign => {op!(a:F64, b:F64 | F64 => {
                            ((a as u64) | ((b as u64) & (1 << 63))) as f64
                        })},
                        Operation::I32WrapI64 => {op!(a:I64 | I32 => a as i32)},
                        Operation::I32TruncSF32 => {op!(a:F32 | I32 => a as i32)},
                        Operation::I32TruncUF32 => {op!(a:F32 | I32 => a as i32)},
                        Operation::I32TruncSF64 => {op!(a:F64 | I32 => a as i32)},
                        Operation::I32TruncUF64 => {op!(a:F64 | I32 => a as i32)},
                        Operation::I64ExtendSI32 => {op!(a:I32 | I64 => a as i64)},
                        Operation::I64ExtendUI32 => {op!(a:I32 | I64 => (a as u32) as i64)},
                        Operation::I64TruncSF32 => {op!(a:F32 | I64 => a as i64)},
                        Operation::I64TruncUF32 => {op!(a:F32 | I64 => a as i64)},
                        Operation::I64TruncSF64 => {op!(a:F64 | I64 => a as i64)},
                        Operation::I64TruncUF64 => {op!(a:F64 | I64 => a as i64)},
                        Operation::F32ConvertSI32 => {op!(a:I32 | F32 => a as f32)},
                        Operation::F32ConvertUI32 => {op!(a:I32 | F32 => (a as u32) as f32)},
                        Operation::F32ConvertSI64 => {op!(a:I64 | F32 => a as f32)},
                        Operation::F32ConvertUI64 => {op!(a:I64 | F32 => (a as u64) as f32)},
                        Operation::F32DemoteF64 => {op!(a:F64 | F32 => a as f32)},
                        Operation::F64ConvertSI32 => {op!(a:I32 | F64 => a as f64)},
                        Operation::F64ConvertUI32 => {op!(a:I32 | F64 => (a as u32) as f64)},
                        Operation::F64ConvertSI64 => {op!(a:I64 | F64 => a as f64)},
                        Operation::F64ConvertUI64 => {op!(a:I64 | F64 => (a as u64) as f64)},
                        Operation::F64PromoteF32 => {op!(a:F32 | F64 => a as f64)},
                        Operation::I32ReinterpretF32 => {op!(a:F32 | I32 => a.to_bits() as i32)},
                        Operation::I64ReinterpretF64 => {op!(a:F64 | I64 => a.to_bits() as i64)},
                        Operation::F32ReinterpretI32 => {op!(a:I32 | F32 => f32::from_bits(a as u32))},
                        Operation::F64ReinterpretI64 => {op!(a:I64 | F64 => f64::from_bits(a as u64))}
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
                        let mut c = CallFrame {
                            tables: & module.tables,
                            types: & module.types,
                            globals: &mut module.globals,
                            memories: &mut module.memories,
                            functions: & module.functions
                        };
                        return module.functions[i](&mut c, args);
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
