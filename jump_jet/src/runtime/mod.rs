use std::io::Read;

use std::collections::HashMap;

use parser::ModuleParser;
use parser::ParseError;

use runtime_tree::ExternalKindInstance;
use runtime_tree::ModuleInstance;
use runtime_tree::ModuleTemplateBuilder;

#[macro_use]
pub mod exports;
pub mod language_types;
pub mod functions;

pub fn instantiate(reader: &mut Read, imports: HashMap<String, HashMap<String, ExternalKindInstance>>) -> Result<ModuleInstance, ParseError> {
    info!("Attempting to parse WebAssembly module");
    let parser = ModuleParser::default();
    parser.parse_module(reader).unwrap().build(imports).unwrap().instantiate()
}