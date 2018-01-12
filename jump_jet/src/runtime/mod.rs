use std::io::Read;

use parse_tree::ParseModule;
use std::collections::HashMap;

use parser::ModuleParser;
use parser::ParseError;

use runtime_tree::ExternalKindInstance;
use runtime_tree::RuntimeModule;
use runtime_tree::RuntimeModuleBuilder;

#[macro_use]
pub mod exports;
pub mod language_types;
pub mod functions;

pub fn instantiate(reader: &mut Read, imports: HashMap<String, HashMap<String, ExternalKindInstance>>) -> Result<RuntimeModule, ParseError> {
    info!("Attempting to parse WebAssembly module");
    let parser = ModuleParser::default();
    parser.parse_module(reader).unwrap().build(imports)
}