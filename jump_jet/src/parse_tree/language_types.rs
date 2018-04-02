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

#[derive(Debug, Clone)]
pub enum ExternalKind {
    Function(usize),
    Table(usize),
    Memory(usize),
    Global(usize),
}

#[derive(Debug)]
pub struct TableType {
    pub elem_type: LanguageType,
    pub limits: ResizableLimits,
}

#[derive(Debug, Clone)]
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
#[derive(PartialEq)]
pub enum Operation {
    // control flow
    Unreachable,
    Nop,
    Block(Block),
    Loop(Block),
    If(Block),
    Else,
    End,
    Branch(i32), // varuint32 | break from block
    BranchIf(i32), // varuint32 | break if condition
    BranchTable(BranchTable), // br_table
    Return, // return

    // callers
    Call(usize), // varuint32
    CallIndirect(usize, bool), // varuint32, reserved

    // parametric
    Drop,
    Select,

    // variable access
    GetLocal(usize), // all varuint32
    SetLocal(usize),
    TeeLocal(usize),
    GetGlobal(usize),
    SetGlobal(usize),

    // Memory related
    I32Load(MemoryImmediate),
    I64Load(MemoryImmediate),
    F32Load(MemoryImmediate),
    F64Load(MemoryImmediate),
    I32Load8S(MemoryImmediate),
    I32Load8U(MemoryImmediate),
    I32Load16S(MemoryImmediate),
    I32Load16U(MemoryImmediate),
    I64Load8S(MemoryImmediate),
    I64Load8U(MemoryImmediate),
    I64Load16S(MemoryImmediate),
    I64Load16U(MemoryImmediate),
    I64Load32S(MemoryImmediate),
    I64Load32U(MemoryImmediate),
    I32Store(MemoryImmediate),
    I64Store(MemoryImmediate),
    F32Store(MemoryImmediate),
    F64Store(MemoryImmediate),
    I32Store8(MemoryImmediate),
    I32Store16(MemoryImmediate),
    I64Store8(MemoryImmediate),
    I64Store16(MemoryImmediate),
    I64Store32(MemoryImmediate),
    CurrentMemory(bool), // varuint1, reserved
    GrowMemory(bool), // varuint1, reserved

    // constants
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),

    // comparisons
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,
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
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,

    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
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
    I32TruncSF32,
    I32TruncUF32,
    I32TruncSF64,
    I32TruncUF64,
    I64ExtendSI32,
    I64ExtendUI32,
    I64TruncSF32,
    I64TruncUF32,
    I64TruncSF64,
    I64TruncUF64,
    F32ConvertSI32,
    F32ConvertUI32,
    F32ConvertSI64,
    F32ConvertUI64,
    F32DemoteF64,
    F64ConvertSI32,
    F64ConvertUI32,
    F64ConvertSI64,
    F64ConvertUI64,
    F64PromoteF32,

    // reinterpretations
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct MemoryImmediate {
    pub flags: u32, // varuint32 - i have no idea what this is
    pub offset: u32,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum BlockType {
    Value(ValueType),
    Empty
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct Block {
    pub block_type: BlockType,
    pub operations: Vec<Operation>
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct BranchTable {
    pub targets: Vec<u32>, // varuint32, possibly change to Vec<BlockType>
    pub default: usize,
}
