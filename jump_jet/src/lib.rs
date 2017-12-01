#[macro_use]
extern crate mopa;

use std::io::Cursor;
use std::fs::File;
use std::error::Error;
use std::path::Path;

//use provider::ProgramProvider;
//use provider::BinaryProvider;

use parser::ParseError;
use tree::Module;

pub use runtime::Runtime;

mod tree;
mod parser;
mod runtime;
mod provider;
