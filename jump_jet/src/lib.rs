#[macro_use]
extern crate log;

pub use runtime::instantiate;

pub mod parse_tree;
pub mod parser;
#[macro_use]
pub mod runtime;
pub mod runtime_tree;
