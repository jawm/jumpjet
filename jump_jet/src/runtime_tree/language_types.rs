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

            (@i $a:expr => $b:ident($c:ty,$d:ty)) => {
                let offset = ($a.flags + $a.offset) as usize;
                let size = mem::size_of::<$c>() as usize;
                let mut a = &stack_frame.data.memories[0].values[offset..offset+8];
                let value = a.read_int::<LittleEndian>(size).unwrap() as $d;
                stack_frame.stack.push(ValueTypeProvider::$b(value));
            };

            (@u $a:expr => $b:ident($c:ty,$d:ty)) => {
                let offset = ($a.flags + $a.offset) as usize;
                let size = mem::size_of::<$c>() as usize;
                let mut a = &stack_frame.data.memories[0].values[offset..offset+8];
                let value = a.read_uint::<LittleEndian>(size).unwrap() as $d;
                stack_frame.stack.push(ValueTypeProvider::$b(value));
            };

            ($a:expr => $b:ident($c:ty)) => {
                mem_op!(@i $a => $b($c,$c));
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
                } else {
                    panic!("if statement didn't have value present to check");
                }
            };
        }

        for operation in &(self.operations) {
            println!("{:?} {:?}", stack_frame.stack, operation);
            match *operation {
                Operation::Unreachable => panic!("Unreachable code executed"),
                Operation::Nop => {},
                Operation::Block(ref b) => {
                    let x = b.execute(stack_frame);
                    if x != 0 { return x-1}
                },
                Operation::Loop(ref b) => {
                    let mut x = 0;
                    while x == 0 {
                        x = b.execute(stack_frame);
                    }
                    if x-1 != 0 {return x-1}
                },
                Operation::If(ref b) => {wasm_if!({
                    b.execute(stack_frame);
                }, {
                    // TODO this searches the entirety of the program for another else
                    // It probably also bugs out if there is a later else part of a different if/else
                    if let Some(index) = b.operations.iter().position(|r|r == &Operation::Else){
                        Block {
                            block_type: b.block_type.clone(),
                            operations: b.operations.clone().split_off(index+1)
                        }.execute(stack_frame);
                    }
                });},
                Operation::Else => {break},
                Operation::End => {break},
                Operation::Branch(b) => {return b},
                Operation::BranchIf(b) => {wasm_if!({return b});},
                Operation::BranchTable(ref b) => {
                    if let Some(ValueTypeProvider::I32(index)) = stack_frame.stack.pop() {
                        let depth = if (index as usize) < b.targets.len() {
                            b.targets[index as usize]-1
                        } else {
                            b.default
                        };
                        return depth;
                    }
                },
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
                    wasm_if!(stack_frame.stack.push(a), stack_frame.stack.push(b));
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
                Operation::F32Load(ref mem) => {
                    let offset = (mem.flags + mem.offset) as usize;
                    let mut a = &stack_frame.data.memories[0].values[offset..offset+8];
                    let value = a.read_f32::<LittleEndian>().unwrap();
                    stack_frame.stack.push(ValueTypeProvider::F32(value));
                },
                Operation::F64Load(ref mem) => {
                    let offset = (mem.flags + mem.offset) as usize;
                    let mut a = &stack_frame.data.memories[0].values[offset..offset+8];
                    let value = a.read_f64::<LittleEndian>().unwrap();
                    stack_frame.stack.push(ValueTypeProvider::F64(value));
                },
                Operation::I32Load8S(ref mem) => {mem_op!(@i mem => I32(i8,i32));},
                Operation::I32Load8U(ref mem) => {mem_op!(@u mem => I32(u8,i32));},
                Operation::I32Load16S(ref mem) => {mem_op!(@i mem => I32(i16,i32));},
                Operation::I32Load16U(ref mem) => {mem_op!(@u mem => I32(u16,i32));},
                Operation::I64Load8S(ref mem) => {mem_op!(@i mem => I64(i8,i64));},
                Operation::I64Load8U(ref mem) => {mem_op!(@u mem => I64(u8,i64));},
                Operation::I64Load16S(ref mem) => {mem_op!(@i mem => I64(i16,i64));},
                Operation::I64Load16U(ref mem) => {mem_op!(@u mem => I64(u16,i64));},
                Operation::I64Load32S(ref mem) => {mem_op!(@i mem => I64(i32,i64));},
                Operation::I64Load32U(ref mem) => {mem_op!(@u mem => I64(u32,i64));},
                Operation::I32Store(ref mem) => {mem_op!(I32(i32) => mem);},
                Operation::I64Store(ref mem) => {mem_op!(I64(i64) => mem);},
                Operation::F32Store(ref mem) => {
                    if let Some(ValueTypeProvider::F32(value)) = stack_frame.stack.pop() {
                        let offset = (mem.flags + mem.offset) as usize;
                        let size = mem::size_of::<f32>() as usize;
                        let mut a = &mut stack_frame.data.memories[0].values[offset..offset+8];
                        a.write_f32::<LittleEndian>(value);
                    } else {
                        panic!("VTP was wrong type or not present!");
                    }
                },
                Operation::F64Store(ref mem) => {
                    if let Some(ValueTypeProvider::F64(value)) = stack_frame.stack.pop() {
                        let offset = (mem.flags + mem.offset) as usize;
                        let size = mem::size_of::<f64>() as usize;
                        let mut a = &mut stack_frame.data.memories[0].values[offset..offset+8];
                        a.write_f64::<LittleEndian>(value);
                    } else {
                        panic!("VTP was wrong type or not present!");
                    }
                },
                Operation::I32Store8(ref mem) => {mem_op!(I32(i8) => mem);},
                Operation::I32Store16(ref mem) => {mem_op!(I32(i16) => mem);},
                Operation::I64Store8(ref mem) => {mem_op!(I64(i8) => mem);},
                Operation::I64Store16(ref mem) => {mem_op!(I64(i16) => mem);},
                Operation::I64Store32(ref mem) => {mem_op!(I64(i32) => mem);},
                Operation::CurrentMemory(_) => {
                    stack_frame.stack.push(ValueTypeProvider::I32(stack_frame.data.memories[0].size()));
                },
                Operation::GrowMemory(_) => {
                    stack_frame.stack.push(ValueTypeProvider::I32(stack_frame.data.memories[0].grow()));
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
    use parse_tree::language_types::BranchTable;
    use parse_tree::language_types::MemoryImmediate;
    use parse_tree::language_types::ResizableLimits;

    // Generates a simple stackframe to work with
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

    macro_rules! block {
        (Value($a:expr), {$($b:expr;)*}) => {
            block!(@ BlockType::Value($a), $($b;)*);
        };

        (Empty, {$($b:expr;)*}) => {
            block!(@ BlockType::Empty, $($b;)*);
        };

        (@ $a:expr, $($b:expr;)*) => {
            Block {
                block_type: $a,
                operations: vec![$($b,)*]
            }
        }
    }

    #[test]
    #[should_panic]
    fn unreachable_panics() {
        sf!(sf);
        let block = block!{ Empty, {
            Operation::Unreachable;
        }};
        block.execute(&mut sf);
    }

    #[test]
    fn nop_does_nothing() {
        sf!(sf);
        let block = block! { Empty, {
            Operation::Nop;
        }};
        block.execute(&mut sf);
        // TODO actually check that the sf hasn't changed?
    }

    #[test]
    fn block_executes() {
        sf!(sf);
        let block = block! { Value(ValueType::I32), {
            Operation::Block(block! { Value(ValueType::I32), {
                Operation::I32Const(1);
            }});
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack.pop(), Some(ValueTypeProvider::I32(1)));
    }

    #[test]
    fn loop_runs_multiple_times() {
        sf!(sf);
        let block = block! { Empty, {
            Operation::I32Const(42);
            Operation::I32Const(0);
            Operation::I32Const(1);
            Operation::I32Const(2);
            Operation::I32Const(3);
            Operation::Loop(block! { Value(ValueType::I32), {
                Operation::I32Eqz;
                Operation::BranchIf(1);
                Operation::End;
            }});
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(42)]);
    }

    #[test]
    fn if_true_i32() {
        sf!(sf);
        let block = block! { Value(ValueType::I32), {
            Operation::I32Const(1);
            Operation::If(block! { Empty, {
                Operation::I32Const(3);
                Operation::I32Const(3);
                Operation::I32Add;
                Operation::Else;
                Operation::End;
            }});
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(6)]);
    }

    #[test]
    fn if_false_else_i32() {
        sf!(sf);
        let block = block! { Value(ValueType::I32), {
            Operation::I32Const(0);
            Operation::If(block! { Empty, {
                Operation::Else;
                Operation:: I32Const(42);
                Operation::End;
            }});
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(42)]);
    }

    #[test]
    fn end_stops_execution() {
        sf!(sf);
        let block = block! { Value(ValueType::I32), {
            Operation::End;
            Operation::I32Const(0);
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![]);
    }

    #[test]
    fn branch_leaves_block() {
        sf!(sf);
        let block = block! { Value(ValueType::I32), {
            Operation::Block(block! { Empty, {
                Operation::I32Const(42);
                Operation::Branch(1);
                Operation::I32Const(13);
                Operation::End;
            }});
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(42)]);
    }

    #[test]
    fn branch_if_true_leaves() {
        sf!(sf);
        let block = block! { Value(ValueType::I32), {
            Operation::Block(block! { Empty, {
                Operation::I32Const(42);
                Operation::I32Const(1);
                Operation::BranchIf(1);
                Operation::I32Const(13);
                Operation::End;
            }});
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(42)]);
    }

    #[test]
    fn branch_if_false_continues() {
        sf!(sf);
        let block = block! { Value(ValueType::I32), {
            Operation::Block(block! { Empty, {
                Operation::I32Const(0);
                Operation::BranchIf(1);
                Operation::I32Const(42);
                Operation::End;
            }});
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(42)]);
    }

    #[test]
    fn branch_table_within_bounds_branches() {
        sf!(sf);
        let block = block! { Value(ValueType::I32), {
            Operation::Block(block! { Empty, {
                Operation::Block(block! { Empty, {
                    Operation::I32Const(42);
                    Operation::I32Const(0);
                    Operation::BranchTable(BranchTable {
                        default: 0,
                        targets: vec![2]
                    });
                    Operation::I32Const(13);
                    Operation::End;
                }});
                Operation::I32Const(13);
                Operation::End;
            }});
            Operation::I32Const(42);
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(42), ValueTypeProvider::I32(42)]);
    }

    #[test]
    fn branch_table_outside_bounds_branches_default() {
        sf!(sf);
        let block = block! { Value(ValueType::I32), {
            Operation::Block(block! { Empty, {
                Operation::Block(block! { Empty, {
                    Operation::I32Const(1);
                    Operation::BranchTable(BranchTable {
                        default: 0,
                        targets: vec![2]
                    });
                    Operation::I32Const(13);
                    Operation::End;
                }});
                Operation::I32Const(42);
                Operation::End;
            }});
            Operation::I32Const(42);
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(42), ValueTypeProvider::I32(42)]);
    }

    // TODO: Call, Return, CallIndirect

    #[test]
    fn drop_value_from_stack() {
        sf!(sf);
        let block = block! { Value(ValueType::I32), {
            Operation::I32Const(0);
            Operation::Drop;
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![]);
    }

    #[test]
    fn select_value_true() {
        sf!(sf);
        let block = block! { Value(ValueType::I32), {
            Operation::I32Const(1);
            Operation::I32Const(13);
            Operation::I32Const(42);
            Operation::Select;
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(42)]);
    }

    #[test]
    fn select_value_false() {
        sf!(sf);
        let block = block! { Value(ValueType::I32), {
            Operation::I32Const(0);
            Operation::I32Const(42);
            Operation::I32Const(13);
            Operation::Select;
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(42)]);
    }

    #[test]
    fn get_local() {
        sf!(sf);
        sf.locals.push(ValueTypeProvider::I32(42));
        let block = block! { Value(ValueType::I32), {
            Operation::GetLocal(0);
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(42)]);
    }

    #[test]
    fn set_local() {
        sf!(sf);
        sf.locals.push(ValueTypeProvider::I32(13));
        let block = block! { Value(ValueType::I32), {
            Operation::I32Const(42);
            Operation::TeeLocal(0);
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(sf.locals, &mut vec![ValueTypeProvider::I32(42)]);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(42)]);
    }

    #[test]
    fn get_global() {
        sf!(sf);
        sf.data.globals.push(ValueTypeProvider::I32(42));
        let block = block! { Value(ValueType::I32), {
            Operation::GetGlobal(0);
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(42)]);
    }

    #[test]
    fn set_global() {
        sf!(sf);
        sf.data.globals.push(ValueTypeProvider::I32(13));
        let block = block! { Value(ValueType::I32), {
            Operation::I32Const(42);
            Operation::SetGlobal(0);
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(&mut *sf.data.globals, &mut vec![ValueTypeProvider::I32(42)]);
    }

    macro_rules! setup_memory {
        ($sf:ident, $start:expr, [$($byte:expr),*]) => {
            $sf.data.memories.push(Memory {
                limits: ResizableLimits {
                    initial: 1,
                    maximum: None
                },
                values: vec![0; 65536]
            });
            let bytes = vec![$($byte),*];
            $sf.data.memories[0].values.splice($start..bytes.len(), bytes);
        };
    }

    #[test]
    fn load_i32_from_memory() {
        {
            sf!(sf);
            setup_memory!(sf, 0, [42]);
            let block = block! { Value(ValueType::I32), {
                Operation::I32Load(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(42)]);
        }
        {
            sf!(sf);
            setup_memory!(sf, 0, [0xef, 0xbe, 0xad, 0xde, 0xff]); // ff isn't read
            let block = block! { Value(ValueType::I32), {
                Operation::I32Load(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(0xdeadbeef)]);
        }
        {
            sf!(sf);
            setup_memory!(sf, 3, [0xef, 0xbe, 0xad, 0xde, 0xff]); // ff isn't read
            let block = block! { Value(ValueType::I32), {
                Operation::I32Load(MemoryImmediate {
                    flags: 0,
                    offset: 3
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(0xdeadbeef)]);
        }
    }

    #[test]
    fn load_i64_from_memory() {
        {
            sf!(sf);
            setup_memory!(sf, 0, [42]);
            let block = block! { Value(ValueType::I32), {
                Operation::I64Load(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I64(42)]);
        }
        {
            sf!(sf);
            setup_memory!(sf, 0, [0xef, 0xbe, 0xad, 0xde, 0xbe, 0xba, 0xfe, 0xca, 0xff]); // ff isn't read
            let block = block! { Value(ValueType::I64), {
                Operation::I64Load(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I64(0xcafebabedeadbeef)]);
        }
    }

    #[test]
    fn load_f32_from_memory() {
        sf!(sf);
        setup_memory!(sf, 0, [0xc3, 0xf5, 0x48, 0x40]);
        let block = block! { Value(ValueType::I32), {
            Operation::F32Load(MemoryImmediate {
                flags: 0,
                offset: 0
            });
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::F32(3.14)]);
    }

    #[test]
    fn load_f64_from_memory() {
        sf!(sf);
        setup_memory!(sf, 0, [0x81, 0xf6, 0x97, 0x9b, 0x77, 0xe3, 0xf9, 0x3f]);
        let block = block! { Value(ValueType::F64), {
            Operation::F64Load(MemoryImmediate {
                flags: 0,
                offset: 0
            });
            Operation::End;
        }};
        block.execute(&mut sf);
        assert_eq!(sf.stack, &mut vec![ValueTypeProvider::F64(1.61803398875)]);
    }

    #[test]
    fn i32_loaders() {
        { // I32Load8S
            sf!(sf);
            setup_memory!(sf, 0, [-42i8 as u8]);
            let block = block! { Value(ValueType::I32), {
                Operation::I32Load8S(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(-42)]);
        }
        { // I32Load8U
            sf!(sf);
            setup_memory!(sf, 0, [42]);
            let block = block! { Value(ValueType::I32), {
                Operation::I32Load8U(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(42)]);
        }
        { // I32Load16S
            sf!(sf);
            setup_memory!(sf, 0, [0x00, 0x83]);
            let block = block! { Value(ValueType::I32), {
                Operation::I32Load16S(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(-32000)]);
        }
        { // I32Load16U
            sf!(sf);
            setup_memory!(sf, 0, [0x00, 0x7d]);
            let block = block! { Value(ValueType::I32), {
                Operation::I32Load16U(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I32(32000)]);
        }
    }

    #[test]
    fn i64_loaders() {
        { // I64Load8S
            sf!(sf);
            setup_memory!(sf, 0, [-42i8 as u8]);
            let block = block! { Value(ValueType::I32), {
                Operation::I64Load8S(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I64(-42)]);
        }
        { // I64Load8U
            sf!(sf);
            setup_memory!(sf, 0, [42]);
            let block = block! { Value(ValueType::I32), {
                Operation::I64Load8U(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I64(42)]);
        }
        { // I64Load16S
            sf!(sf);
            setup_memory!(sf, 0, [0x00, 0x83]);
            let block = block! { Value(ValueType::I32), {
                Operation::I64Load16S(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I64(-32000)]);
        }
        { // I64Load16U
            sf!(sf);
            setup_memory!(sf, 0, [0x00, 0x7d]);
            let block = block! { Value(ValueType::I32), {
                Operation::I64Load16U(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I64(32000)]);
        }
        { // I64Load32S
            sf!(sf);
            setup_memory!(sf, 0, [0x2e, 0xfd, 0x69, 0xb6]);
            let block = block! { Value(ValueType::I32), {
                Operation::I64Load32S(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I64(-1234567890)]);
        }
        { // I64Load32U
            sf!(sf);
            setup_memory!(sf, 0, [0xD2, 0x02, 0x96, 0x49]);
            let block = block! { Value(ValueType::I32), {
                Operation::I64Load32U(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.stack, &mut vec![ValueTypeProvider::I64(1234567890)]);
        }
    }

    #[test]
    fn basic_number_store_ops() {
        { // I32Store
            sf!(sf);
            setup_memory!(sf, 0, []);
            let block = block! { Value(ValueType::I32), {
                Operation::I32Const(1234567890);
                Operation::I32Store(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.data.memories[0].values[0..5], [0xD2, 0x02, 0x96, 0x49, 0x00]);
        }
        { // I64Store
            sf!(sf);
            setup_memory!(sf, 0, []);
            let block = block! { Value(ValueType::I32), {
                Operation::I64Const(0x123456789abcdef0);
                Operation::I64Store(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.data.memories[0].values[0..9], [0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12, 0x00]);
        }
        { // F32Store
            sf!(sf);
            setup_memory!(sf, 0, []);
            let block = block! { Value(ValueType::I32), {
                Operation::F32Const(3.1415);
                Operation::F32Store(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.data.memories[0].values[0..5], [0x56, 0x0e, 0x49, 0x40, 0x00]);
        }
        { // F64Store
            sf!(sf);
            setup_memory!(sf, 0, []);
            let block = block! { Value(ValueType::I32), {
                Operation::F64Const(1.61803398875);
                Operation::F64Store(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.data.memories[0].values[0..9], [0x81, 0xf6, 0x97, 0x9b, 0x77, 0xe3, 0xf9, 0x3f, 0x00]);
        }
    }

    #[test]
    fn test_complex_int_store_ops() {
        { // I32Store8

            sf!(sf);
            setup_memory!(sf, 0, []);
            let block = block! { Value(ValueType::I32), {
                Operation::I32Const(42);
                Operation::I32Store8(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.data.memories[0].values[0..2], [42, 0x00]);
        }
        { // I32Store8
            sf!(sf);
            setup_memory!(sf, 0, []);
            let block = block! { Value(ValueType::I32), {
                Operation::I32Const(0xff42);
                Operation::I32Store8(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.data.memories[0].values[0..2], [0x42, 0x00]);
        }
        { // I32Store16
            sf!(sf);
            setup_memory!(sf, 0, []);
            let block = block! { Value(ValueType::I32), {
                Operation::I32Const(0xbeef);
                Operation::I32Store(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.data.memories[0].values[0..3], [0xef, 0xbe, 0x00]);
        }
        { // I64Store8
            sf!(sf);
            setup_memory!(sf, 0, []);
            let block = block! { Value(ValueType::I32), {
                Operation::I64Const(42);
                Operation::I64Store(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.data.memories[0].values[0..2], [42, 0x00]);
        }
        { // I64Store16
            sf!(sf);
            setup_memory!(sf, 0, []);
            let block = block! { Value(ValueType::I32), {
                Operation::I64Const(0xbeef);
                Operation::I64Store(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.data.memories[0].values[0..3], [0xef, 0xbe, 0x00]);
        }
        { // I64Store32
            sf!(sf);
            setup_memory!(sf, 0, []);
            let block = block! { Value(ValueType::I32), {
                Operation::I64Const(1234567890);
                Operation::I64Store(MemoryImmediate {
                    flags: 0,
                    offset: 0
                });
                Operation::End;
            }};
            block.execute(&mut sf);
            assert_eq!(sf.data.memories[0].values[0..5], [0xD2, 0x02, 0x96, 0x49, 0x00]);
        }
    }
}