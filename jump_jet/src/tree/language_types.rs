#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub enum LanguageType {
    Value(ValueType),
    Anyfunc, // no static signature validation check
    Func,
    EmptyBlock,
}

#[derive(Debug)]
pub enum ExternalKind {
    Function(usize), // possibly have it go into the types section, instead of storing index
    Table(usize),
    Memory(usize),
    Global(usize),
}

#[derive(Debug)]
pub struct TableType {
    pub elem_type: LanguageType,
    pub limits: ResizableLimits,
}

#[derive(Debug)]
pub struct ResizableLimits {
    pub initial: u64,
    pub maximum: Option<u64>,
}

#[derive(Debug)]
pub struct GlobalType {
    pub content_type: ValueType,
    pub mutability: bool,
}

#[derive(Debug)]
pub enum InitExpression {
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),
    GetGlobal(usize),
}

#[derive(Clone)]
#[derive(Debug)]
pub enum Operation {
    // control flow
    Unreachable,
    Nop,
    Block(BlockType),
    Loop(BlockType), // loop
    If(BlockType), // if
    Else, // else - TODO should have BlockType immediate?
    End,
    Branch(u32), // varuint32 | break from block
    BranchIf(u32), // varuint32 | break if condition
    BranchTable(BranchTable), // br_table
    Return, // return

    // callers
    Call(u64), // varuint32
    CallIndirect(usize, bool), // varuint32, reserved

    // parametric
    Drop,
    Select,

    // variable access
    GetLocal(u32), // all varuint32
    SetLocal(u64),
    TeeLocal(u64),
    GetGlobal(u64),
    SetGlobal(u64),

    // Memory related
    I32Load(MemoryImmediate),
    I64Load(MemoryImmediate),
    F32Load(MemoryImmediate),
    F64Load(MemoryImmediate),
    I32Load8(MemoryImmediate),
    U32Load8(MemoryImmediate),
    I32Load16(MemoryImmediate),
    U32Load16(MemoryImmediate),
    I64Load8(MemoryImmediate),
    U64Load8(MemoryImmediate),
    I64Load16(MemoryImmediate),
    U64Load16(MemoryImmediate),
    I64Load32(MemoryImmediate),
    U64Load32(MemoryImmediate),
    I32Store(MemoryImmediate),
    I64Store(MemoryImmediate),
    F32Store(MemoryImmediate),
    F64Store(MemoryImmediate),
    I32Store8(MemoryImmediate),
    I32Store16(MemoryImmediate),
    I64Store8(MemoryImmediate),
    I64Store16(MemoryImmediate),
    I64Store32(MemoryImmediate),
    CurrentMemory(u64), // varuint1, reserved
    GrowMemory(u64), // varuint1, reserved

    // constants
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),

    // comparisons
    I32Eqz,
    I32Eq,
    I32Ne,
    I32Lt,
    U32Lt,
    I32Gt,
    U32Gt,
    I32Le,
    U32Le,
    I32Ge,
    U32Ge,

    I64Eqz,
    I64Eq,
    I64Ne,
    I64Lt,
    U64Lt,
    I64Gt,
    U64Gt,
    I64Le,
    U64Le,
    I64Ge,
    U64Ge,

    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,

    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,

    // numeric
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32Div,
    U32Div,
    I32Rem,
    U32Rem,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32Shr,
    U32Shr,
    I32Rotl,
    I32Rotr,

    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64Div,
    U64Div,
    I64Rem,
    U64Rem,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64Shr,
    U64Shr,
    I64Rotl,
    I64Rotr,

    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,

    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,

    // conversions
    I32WrapI64,
    I32TruncF32,
    U32TruncF32,
    I32TruncF64,
    U32TruncF64,
    I64ExtendI32,
    U64ExtendI32,
    I64TruncF32,
    U64TruncF32,
    I64TruncF64,
    U64TruncF64,
    F32ConvertSI32,
    F32ConvertUI32,
    F32ConvertSI64,
    F32ConvertUI64,
    F32DemoteF64,
    F64ConvertSI32,
    F64ConvertUI32,
    F64PromoteF32,

    // reinterpretations
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,
}

// TODO remove this annotation boi
#[allow(dead_code)]
#[derive(Clone)]
#[derive(Debug)]
pub struct MemoryImmediate {
    flags: u64, // varuint32 - i have no idea what this is
    offset: u64, // varuint32
}

#[derive(Clone)]
#[derive(Debug)]
pub enum BlockType {
    Value(ValueType),
    Empty
}

#[derive(Clone)]
#[derive(Debug)]
pub struct BranchTable {
    pub targets: Vec<u32>, // varuint32, possibly change to Vec<BlockType>
    pub default: usize,
}
