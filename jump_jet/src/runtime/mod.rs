use std::io::prelude::*;
use tree::language_types::ValueType;
use tree::Module;
use std::collections::HashMap;
use std::io::Cursor;
use std::fs::File;
use std::error::Error;
use std::path::Path;

use parser::ParseError;
use parser::ModuleParserInfo;

pub struct Runtime {
    exposed: HashMap<
        String, 
        HashMap<
            String,
            fn(Vec<ValueType>) -> Vec<ValueType>
        >
    >,
    modules: HashMap<String, Module>,
    parser: ModuleParserInfo
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime {
            exposed: HashMap::new(),
            modules: HashMap::new(),
            parser: ModuleParserInfo::default()
        }
    }

    pub fn expose(&mut self, namespace: String, functions: Vec<fn(Vec<ValueType>)->Vec<ValueType>>) {
        println!("exposing functions under a namespace");
    }

    pub fn add_module(&mut self, name: String, path: String) {
        println!("loading the module into the runtime");
        match build_module(&path) {
            Ok(module) => {self.modules.insert(name, module);},
            Err(error) => println!("Error parsing")
        };
    }

    pub fn prepare(&mut self, name: String) {
        println!("preparing the given thingy");
    }

    pub fn get(&self, name: String) -> Option<& Module> {
        self.modules.get(&name)
    }
}

impl Module {
    pub fn run(&self) {
        println!("running the module yo");
    }
}

fn build_module(file_name: &str) -> Result<Module, ParseError> {
    println!("Attempting to read file: {}",file_name);
    let path = Path::new(file_name);
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };
    let mut buffer = vec![];
    // read the whole file
    file.read_to_end(&mut buffer).unwrap();
    
    Module::parse(&mut Cursor::new(&buffer[..]))
}