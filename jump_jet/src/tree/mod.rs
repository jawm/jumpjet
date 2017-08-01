use std::io::Read;

pub mod section;
mod types;
mod language_types;
mod imports;
mod functions;
mod tables;
mod memory;
mod globals;
mod exports;
mod start;
mod elements;
mod code;
mod data;

pub struct Module {
	pub sections: Vec<section::Section>,
	pub version: u32
}