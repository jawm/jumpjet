use std::collections::HashMap;

pub mod types;
pub mod language_types;
pub mod memory;
pub mod functions;
pub mod tables;
pub mod globals;

#[derive(Debug)]
pub struct ParseModule {
    pub version: u32,
    pub types: Vec<types::TypeDefinition>,
    pub imports: HashMap<String, HashMap<String, language_types::ExternalKind>>,
    pub function_signatures: Vec<usize>,
    pub function_bodies: Vec<functions::FuncBody>,
    pub tables: Vec<tables::Table>,
    pub memories: Vec<memory::Memory>,
    pub globals: Vec<globals::Global>,
    pub exports: HashMap<String, language_types::ExternalKind>,
    pub start_function: Option<usize>,
}
