use std::collections::HashMap;

pub mod section;
pub mod types;
pub mod language_types;
pub mod imports;
pub mod functions;
pub mod tables;
pub mod memory;
pub mod globals;
pub mod exports;
pub mod start;
pub mod elements;
pub mod code;
pub mod data;

pub struct Module<'a> {
    pub version: u32,
    pub functions: Vec<functions::Function>,
    pub imports: HashMap<&'a str, HashMap<&'a str, language_types::ExternalKind>>
}