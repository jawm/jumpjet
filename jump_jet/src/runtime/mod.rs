use std::io::Read;

use tree::language_types::ValueType;
use tree::Module;
use std::collections::HashMap;
use std::fs::File;
use std::error::Error;
use std::path::Path;

use parser::ModuleParser;
use parser::ParseError;

#[macro_use]
pub mod exports;
pub mod language_types;
pub mod functions;

pub fn instantiate(reader: &mut Read, imports: HashMap<String, HashMap<String, u32>>) -> Result<Module, ParseError> {
    let parser = ModuleParser::default();
    parser.parse_module(reader)
}




pub struct Runtime {
    modules: HashMap<String, Module>,
    parser: ModuleParser
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime {
            // exposed: HashMap::new(),
            modules: HashMap::new(),
            parser: ModuleParser::default()
        }
    }

    pub fn expose(&mut self, _namespace: &str, _functions: Vec<fn(Vec<ValueType>)->Vec<ValueType>>) {
        println!("Exposing functions under a namespace");
    }

    pub fn add_module(&mut self, name: &str, path: &str) {
        println!("Loading the module into the runtime");
        let path = Path::new(path);
        let display = path.display();
        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why.description()),
            Ok(file) => file,
        };

        let module: Module = match self.parser.parse_module(&mut file) {
            Ok(module) => module,
            Err(err) => panic!("Failed to parse module: {:?}", err)
        };
        
        self.modules.insert(name.to_string(), module);
    }

    pub fn prepare(&mut self, _name: &str) {
        println!("Preparing module");

    }

    pub fn get(&self, name: &str) -> Option<& Module> {
        self.modules.get(name)
    }
}

impl Module {
    pub fn run(&self) {
        println!("running the module yo");
    }
}