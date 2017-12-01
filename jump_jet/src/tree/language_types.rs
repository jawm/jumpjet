// TODO switch from enums to a bunch of structs instead.
use std::clone::Clone;

macro_rules! language_type {
    ($name:ident) => {
        pub struct $name {}
        impl LanguageType for $name {}
    }
}

macro_rules! language_types {
    ($($name:ident);*) => {
        $(
            language_type!($name);
        )*
    }
}

macro_rules! value_type {
    ($name:ident) => {
        language_type!($name);
        impl ValueType for $name {}
    }
}

macro_rules! value_types {
    ($($name:ident);*) => {
        $(
            value_type!($name);
        )*
    }
}

pub trait LanguageType {}
pub trait ValueType : LanguageType {}

value_types!(
    i_32;
    i_64;
    f_32;
    f_64);
language_types!(
    anyfunc;
    func;
    empty_block);

// macro_rules! language_types {
//     ($($name:ident);*;$()*) => {
//         $(
//             language_type!($name);
//         )*
//     }
// }

// language_types!(
//     anyfunc;
//     func;
//     empty_block;
//     value_types!(
//         I32;
//         I64;
//         F32;
//         F64;
//     );
// );




// #[derive(Debug)]
// #[derive(Clone)]
// pub enum ValueType {
//     i_32,
//     i_64,
//     f_32,
//     f_64,
// }

// #[derive(PartialEq)]
// #[derive(Debug)]
// #[derive(Clone)]
// pub enum LanguageType {
//     i_32,
//     i_64,
//     f_32,
//     f_64,
//     anyfunc, // no static signature validation check
//     func,
//     empty_block,
// }

#[derive(Debug)]
pub enum ExternalKind {
    function(u64), // possibly have it go into the types section, instead of storing index
    table(u64),
    memory(u64),
    global(u64),
}

#[derive(Debug)]
pub struct TableType {
    pub elemType: i64,
    pub limits: ResizableLimits,
}

#[derive(Debug)]
pub struct ResizableLimits {
    pub initial: u64,
    pub maximum: Option<u64>,
}

#[derive(Debug)]
pub struct GlobalType {
    pub contentType: ValueType,
    pub mutability: bool,
}

#[derive(Debug)]
pub enum InitExpressions {
    i32_const(i64),
    i64_const(i64),
    f32_const(u64),
    f64_const(u64),
    get_global(u64),
}

#[derive(Debug)]
pub enum Operation {
    // control flow
    unreachable,
    nop,
    block(BlockType),
    repeated(BlockType), // loop
    condition(BlockType), // if
    not_condition, // else - TODO should have BlockType immediate?
    end,
    escape(u64), // varuint32 | break from block
    escape_if(u64), // varuint32 | break if condition
    branch_table(BranchTable), // br_table
    return_value, // return

    // callers
    call(u64), // varuint32
    call_indirect(u64), // varuint32, reserved

    // parametric
    drop,
    select,

    // variable access
    get_local(u64), // all varuint32
    set_local(u64),
    tee_local(u64),
    get_global(u64),
    set_global(u64),

    // Memory related
    i32_load(MemoryImmediate),
    i64_load(MemoryImmediate),
    f32_load(MemoryImmediate),
    f64_load(MemoryImmediate),
    i32_load_8(MemoryImmediate),
    u32_load_8(MemoryImmediate),
    i32_load_16(MemoryImmediate),
    u32_load_16(MemoryImmediate),
    i64_load_8(MemoryImmediate),
    u64_load_8(MemoryImmediate),
    i64_load_16(MemoryImmediate),
    u64_load_16(MemoryImmediate),
    i64_load_32(MemoryImmediate),
    u64_load_32(MemoryImmediate),
    i32_store(MemoryImmediate),
    i64_store(MemoryImmediate),
    f32_store(MemoryImmediate),
    f64_store(MemoryImmediate),
    i32_store_8(MemoryImmediate),
    i32_store_16(MemoryImmediate),
    i64_store_8(MemoryImmediate),
    i64_store_16(MemoryImmediate),
    i64_store_32(MemoryImmediate),
    current_memory(u64), // varuint1, reserved
    grow_memory(u64), // varuint1, reserved

    // constants
    i32_const,
    i64_const,
    f32_const,
    f64_const,

    // comparisons
    i32_eqz,
    i32_eq,
    i32_ne,
    i32_lt,
    u32_lt,
    i32_gt,
    u32_gt,
    i32_le,
    u32_le,
    i32_ge,
    u32_ge,

    i64_eqz,
    i64_eq,
    i64_ne,
    i64_lt,
    u64_lt,
    i64_gt,
    u64_gt,
    i64_le,
    u64_le,
    i64_ge,
    u64_ge,

    f32_eq,
    f32_ne,
    f32_lt,
    f32_gt,
    f32_le,
    f32_ge,

    f64_eq,
    f64_ne,
    f64_lt,
    f64_gt,
    f64_le,
    f64_ge,

    // numeric
    i32_clz,
    i32_ctz,
    i32_popcnt,
    i32_add,
    i32_sub,
    i32_mul,
    i32_div,
    u32_div,
    i32_rem,
    u32_rem,
    i32_and,
    i32_or,
    i32_xor,
    i32_shl,
    i32_shr,
    u32_shr,
    i32_rotl,
    i32_rotr,

    i64_clz,
    i64_ctz,
    i64_popcnt,
    i64_add,
    i64_sub,
    i64_mul,
    i64_div,
    u64_div,
    i64_rem,
    u64_rem,
    i64_and,
    i64_or,
    i64_xor,
    i64_shl,
    i64_shr,
    u64_shr,
    i64_rotl,
    i64_rotr,

    f32_abs,
    f32_neg,
    f32_ceil,
    f32_floor,
    f32_trunc,
    f32_nearest,
    f32_sqrt,
    f32_add,
    f32_sub,
    f32_mul,
    f32_div,
    f32_min,
    f32_max,
    f32_copysign,

    f64_abs,
    f64_neg,
    f64_ceil,
    f64_floor,
    f64_trunc,
    f64_nearest,
    f64_sqrt,
    f64_add,
    f64_sub,
    f64_mul,
    f64_div,
    f64_min,
    f64_max,
    f64_copysign,

    // conversions
    i32_wrap_i64,
    i32_trunc_f32,
    u32_trunc_f32,
    i32_trunc_f64,
    u32_trunc_f64,
    i64_extend_i32,
    u64_extend_i32,
    i64_trunc_f32,
    u64_trunc_f32,
    i64_trunc_f64,
    u64_trunc_f64,
    f32_convert_s_i32,
    f32_convert_u_i32,
    f32_convert_s_i64,
    f32_convert_u_i64,
    f32_demote_f64,
    f64_convert_s_i32,
    f64_convert_u_i32,
    f64_promote_f32,

    // reinterpretations
    i32_reinterpret_f32,
    i64_reinterpret_f64,
    f32_reinterpret_i32,
    f64_reinterpret_i64,
}

#[derive(Debug)]
pub struct MemoryImmediate {
    flags: u64, // varuint32 - i have no idea what this is
    offset: u64, // varuint32
}

#[derive(Debug)]
pub struct BlockType {
    signature: i64, // varint7 - either 0x40 or a ValueType
}

#[derive(Debug)]
pub struct BranchTable {
    targets: Vec<u64>, // varuint32, possibly change to Vec<BlockType>
    default: u64,
}
