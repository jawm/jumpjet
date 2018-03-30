use parse_tree::language_types::Block;
use parse_tree::language_types::Operation;
use parse_tree::language_types::ValueType;
use parse_tree::memory::Memory;
use parse_tree::tables::Table;
use parse_tree::types::TypeDefinition;

use runtime_tree::byteorder::LittleEndian;
use runtime_tree::byteorder::ReadBytesExt;
use runtime_tree::byteorder::WriteBytesExt;
//use runtime_tree::RuntimeModule;
use runtime_tree::ModuleInstanceData;
use runtime_tree::Func;

use std::cell::RefMut;
use std::mem;

pub enum ExternalKindInstance {
    Function(Func),
    Table(usize),
    Memory(usize),
    Global(usize),
}

pub enum Import {
    //TODO imported functions probably shouldn't get access to ModuleInstanceData. Wrap it in another closure that swallows that.
    Function(Box<Fn(&mut ModuleInstanceData, Vec<ValueTypeProvider>)->Vec<ValueTypeProvider>>),
    Table(usize),
    Memory(usize),
    Global(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueTypeProvider {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

pub struct StackFrame<'b, 'a: 'b> {
    pub data: &'b mut ModuleInstanceData<'a>,
    pub locals: &'b mut Vec<ValueTypeProvider>,
    pub stack: &'b mut Vec<ValueTypeProvider>
}

pub trait Execute {
    fn execute(&self, &mut StackFrame) -> i32;
}

impl Execute for Block {
    fn execute(&self, stack_frame: &mut StackFrame) -> i32 {

        println!("Executing operations");
        let stack_size = stack_frame.stack.len();

        macro_rules! op {
            ($($a:ident:$b:ident),* | $r:ident => $op:expr) => {
                op!(@a stack_frame.stack.push(ValueTypeProvider::$r($op)), $($a:$b),*);
            };

            ($($a:ident:$b:ident),* | @bool => $op:expr) => {
                op!(@a stack_frame.stack.push(ValueTypeProvider::I32($op as i32)), $($a:$b),*);
            };

            ($($a:ident:$b:ident),* | @any => $op:expr) => {
                op!(@a stack_frame.stack.push($op), $($a:$b),*);
            };

            (@a $op:expr, $hn:ident:$ht:ident, $($a:ident:$b:ident),*) => {
                if let Some(ValueTypeProvider::$ht($hn)) = stack_frame.stack.pop() {
                    op!(@a $op, $($a:$b),*);
                }
            };

            (@a $op:expr, $hn:ident:$ht:ident) => {
                if let Some(ValueTypeProvider::$ht($hn)) = stack_frame.stack.pop() {
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
                let size = mem::size_of::<$c>() as usize;
                let mut a = &stack_frame.data.memories[0].values[offset..offset+8];
                let value = a.read_int::<LittleEndian>(size).unwrap() as $d;
                stack_frame.stack.push(ValueTypeProvider::$b(value));
            };

            ($a:expr => $b:ident($c:ty)) => {
                mem_op!($a => $b($c,$c));
            };

            ($a:ident($c:ty) => $d:expr) => {
                if let Some(ValueTypeProvider::$a(value)) = stack_frame.stack.pop() {
                    let offset = ($d.flags + $d.offset) as usize;
                    let size = mem::size_of::<$c>() as usize;
                    let mut a = &mut stack_frame.data.memories[0].values[offset..offset+8];
                    a.write_int::<LittleEndian>(value as $c as i64, size);
                } else {
                    panic!("VTP was wrong type or not present!");
                }
            };

        }

        macro_rules! wasm_if {
            ($truthy:expr) => {
                wasm_if!($truthy, {});
            };

            ($truthy:expr, $falsey:expr) => {
                if let Some(ValueTypeProvider::I32(i)) = stack_frame.stack.pop() {
                    println!("{:?}", i);
                    if i != 0 {
                        println!("truthy {:?}", i);
                        $truthy;
                    } else {
                        println!("falsey {:?}", i);
                        $falsey;
                    }
                }
            };
        }

        for operation in &(self.operations) {
            println!("{:?} ", operation);
            match *operation {
                Operation::Unreachable => panic!("Unreachable code executed"),
                Operation::Nop => {},
                Operation::Block(ref b) => {
                    let x = b.execute(stack_frame);
                    if x != 0 { return x-1}
                },
                Operation::Loop(ref b) => {},
                Operation::If(ref b) => {println!("calling if"); wasm_if!({
                    b.execute(stack_frame);
                }, {
                    let index = b.operations.iter().position(|r|r == &Operation::Else).unwrap();
                    Block {
                        block_type: b.block_type.clone(),
                        operations: b.operations.clone().split_off(index+1)
                    }.execute(stack_frame);
                });},
                Operation::Else => {break},
                Operation::End => {break},
                Operation::Branch(b) => {return b},
                Operation::BranchIf(b) => {wasm_if!({return b});},
                Operation::BranchTable(ref b) => {break;},
                Operation::Call(index) => {
                    let data = &mut stack_frame.data;
                    let function = data.functions.get(index).unwrap();
                    let mut args = vec![];
                    for param in &(function.signature.parameters) {
                        match *param {
                            ValueType::I32 => {
                                if let Some(ValueTypeProvider::I32(v)) = stack_frame.stack.pop() {
                                    args.push(ValueTypeProvider::I32(v));
                                } else {
                                    panic!("wrong argument type");
                                }
                            },
                            ValueType::I64 => {
                                if let Some(ValueTypeProvider::I64(v)) = stack_frame.stack.pop() {
                                    args.push(ValueTypeProvider::I64(v));
                                } else {
                                    panic!("wrong argument type");
                                }
                            },
                            ValueType::F32 => {
                                if let Some(ValueTypeProvider::F32(v)) = stack_frame.stack.pop() {
                                    args.push(ValueTypeProvider::F32(v));
                                } else {
                                    panic!("wrong argument type");
                                }
                            },
                            ValueType::F64 => {
                                if let Some(ValueTypeProvider::F64(v)) = stack_frame.stack.pop() {
                                    args.push(ValueTypeProvider::F64(v));
                                } else {
                                    panic!("wrong argument type");
                                }
                            },
                        }
                    }
                    println!("{:?}", function.signature);
                    for ValueTypeProvider in (function.callable)(data, args) {
                        stack_frame.stack.push(ValueTypeProvider);
                    }
                },
                Operation::Return => {return -1;}, //TODO FIX THIS !! TODO TODO TODO
                Operation::CallIndirect(idx, _) => {
                    let data = &mut stack_frame.data;
                    let TypeDefinition::Func(ref signature) = data.types[idx].clone();
                    let mut args = vec![];
                    for param in &(signature.parameters) {
                        match *param {
                            ValueType::I32 => {
                                if let Some(ValueTypeProvider::I32(v)) = stack_frame.stack.pop() {
                                    args.push(ValueTypeProvider::I32(v));
                                } else {
                                    panic!("wrong argument type");
                                }
                            },
                            ValueType::I64 => {
                                if let Some(ValueTypeProvider::I64(v)) = stack_frame.stack.pop() {
                                    args.push(ValueTypeProvider::I64(v));
                                } else {
                                    panic!("wrong argument type");
                                }
                            },
                            ValueType::F32 => {
                                if let Some(ValueTypeProvider::F32(v)) = stack_frame.stack.pop() {
                                    args.push(ValueTypeProvider::F32(v));
                                } else {
                                    panic!("wrong argument type");
                                }
                            },
                            ValueType::F64 => {
                                if let Some(ValueTypeProvider::F64(v)) = stack_frame.stack.pop() {
                                    args.push(ValueTypeProvider::F64(v));
                                } else {
                                    panic!("wrong argument type");
                                }
                            },
                        }
                    }
                    if let Some(ValueTypeProvider::I32(index)) = stack_frame.stack.pop() {
                        let fn_index = {
                            let &Table::AnyFunc{ref limits, ref values} = &(data.tables)[0];
                            values.get(index as usize).unwrap().clone()
                        };
                        let callable = &data.functions.get(fn_index).unwrap().callable;
                        for ValueTypeProvider in callable(data, args) {
                            stack_frame.stack.push(ValueTypeProvider);
                        }
                    } else {
                        panic!("function not found or not indexed by i32");
                    }
                },
                Operation::Drop => {stack_frame.stack.pop();},
                Operation::Select => {
                    let a = stack_frame.stack.pop().unwrap();
                    let b = stack_frame.stack.pop().unwrap();
                    if mem::discriminant(&a) != mem::discriminant(&b) {
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
                Operation::GetLocal(idx) => stack_frame.stack.push(stack_frame.locals[idx].clone()),
                Operation::SetLocal(idx) => {
                    if let Some(vtp) = stack_frame.stack.pop() {
                        if mem::discriminant(&stack_frame.locals[idx]) == mem::discriminant(&vtp) {
                            stack_frame.locals[idx] = vtp;
                        } else {
                            panic!("Wrong type provided for local set");
                        }
                    } else {
                        panic!("no values on stack");
                    }
                },
                Operation::TeeLocal(idx) => {
                    if let Some(vtp) = stack_frame.stack.pop() {
                        if mem::discriminant(&stack_frame.locals[idx]) == mem::discriminant(&vtp) {
                            stack_frame.locals[idx] = vtp.clone();
                            stack_frame.stack.push(vtp);
                        } else {
                            panic!("Wrong type provided for local set");
                        }
                    } else {
                        panic!("no values on stack");
                    }
                },
                Operation::GetGlobal(idx) => stack_frame.stack.push(stack_frame.data.globals[idx].clone()),
                Operation::SetGlobal(idx) => {
                    if let Some(vtp) = stack_frame.stack.pop() {
                        if mem::discriminant(&stack_frame.data.globals[idx]) == mem::discriminant(&vtp) {
                            stack_frame.data.globals[idx] = vtp;
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
                Operation::CurrentMemory(_) => {
                    stack.push(ValueTypeProvider::I32(module.memories[0].size()));
                },
                Operation::GrowMemory(_) => {
                    stack.push(ValueTypeProvider::I32(module.memories[0].grow()));
                },
                Operation::I32Const(value) => {stack_frame.stack.push(ValueTypeProvider::I32(value))},
                Operation::I64Const(value) => {stack_frame.stack.push(ValueTypeProvider::I64(value))},
                Operation::F32Const(value) => {stack_frame.stack.push(ValueTypeProvider::F32(value))},
                Operation::F64Const(value) => {stack_frame.stack.push(ValueTypeProvider::F64(value))},
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
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use parse_tree::language_types::BlockType;

    macro_rules! sf {
        ($a:ident) => {
            let functions = vec![];
            let globals = RefCell::new(vec![]);
            let memories = RefCell::new(vec![]);
            let tables = RefCell::new(vec![]);
            let mut $a = StackFrame {
                data: &mut ModuleInstanceData {
                    functions: &functions,
                    globals: globals.borrow_mut(),
                    memories: memories.borrow_mut(),
                    tables: tables.borrow_mut(),
                    types: vec![]
                },
                locals: &mut vec![],
                stack: &mut vec![]
            };
        }
    }

    #[test]
    #[should_panic]
    fn unreachable_panics() {
        sf!(sf);
        let block = Block {
            block_type: BlockType::Value(ValueType::I32),
            operations: vec![Operation::Unreachable]
        };
        block.execute(&mut sf);
    }


    #[test]
    fn if_true_i32() {
        sf!(sf);
        let block = Block {
            block_type: BlockType::Value(ValueType::I32),
            operations: vec![
                Operation::I32Const(1),
                Operation::If(Block {
                    block_type: BlockType::Empty,
                    operations: vec![
                        Operation::I32Const(3),
                        Operation::I32Const(3),
                        Operation::I32Add,
                        Operation::Else,
                        Operation::End
                    ]
                }),
                Operation::End
            ]
        };
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(6)]);
    }

    #[test]
    fn if_false_else_i32() {
        sf!(sf);
        let block = Block {
            block_type: BlockType::Value(ValueType::I32),
            operations: vec![
                Operation::I32Const(0),
                Operation::If(Block {
                    block_type: BlockType::Empty,
                    operations: vec![
                        Operation::Else,
                        Operation:: I32Const(42),
                        Operation::End
                    ]
                }),
                Operation::End
            ]
        };
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(42)]);
    }

}