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

#[derive(Debug)]
pub struct Module {
    pub version: u32,
    pub types: Vec<types::TypeDefinition>,
    pub imports: HashMap<String, HashMap<String, language_types::ExternalKind>>,
    pub functions: Vec<functions::Function>,
    pub tables: Vec<tables::Table>,
    pub memories: Vec<memory::Memory>,
    pub globals: Vec<globals::Global>,
    pub exports: HashMap<String, language_types::ExternalKind>,
    pub start_function: Option<functions::Function>,
    // code
}
