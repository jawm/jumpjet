use std::io::prelude::*;
use std::io::Cursor;
use std::fs::File;
use std::error::Error;
use std::path::Path;

use provider::ProgramProvider;
use provider::BinaryProvider;

mod tree;
mod parser;

mod provider;

pub fn build_module(file_name: &str) {
    println!("Attempting to read file: {}",file_name);
    let path = Path::new(file_name);
    let display = path.display();
    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open {}: {}", display,
                                                   why.description()),
        Ok(file) => file,
    };
    let mut buffer = vec![];
    // read the whole file
    file.read_to_end(&mut buffer).unwrap();
    
    tree::Module::parse(&mut Cursor::new(&buffer[..]));

    // BinaryProvider::new(buffer).provide();
}


