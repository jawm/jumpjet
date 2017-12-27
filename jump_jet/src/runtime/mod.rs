use std::io::prelude::*;
use tree::language_types::ValueType;
use tree::Module;
use std::collections::HashMap;
use std::io::Cursor;
use std::fs::File;
use std::error::Error;
use std::path::Path;


use parser::ModuleParser;

pub struct Runtime<'a> {
    exposed: HashMap<
        String, 
        HashMap<
            String,
            fn(Vec<ValueType>) -> Vec<ValueType>
        >
    >,
    modules: HashMap<String, Module<'a>>,
    parser: ModuleParser
}

impl<'a> Runtime<'a> {
    pub fn new() -> Runtime<'a> {
        Runtime {
            exposed: HashMap::new(),
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
        let mut buffer = vec![];
        
        file.read_to_end(&mut buffer).unwrap();
        let mut reader = Cursor::new(&buffer[..]);

        let module: Module<'a> = match self.parser.parse_module(&mut reader) {
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

impl<'a> Module<'a> {
    pub fn run(&self) {
        println!("running the module yo");
    }
}