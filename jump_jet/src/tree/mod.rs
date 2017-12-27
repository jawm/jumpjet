use std::collections::HashMap;

pub mod section;
pub mod types;
pub mod language_types;
pub mod imports;
pub mod functions;
pub mod tables;
pub mod memory;
pub mod globals;
pub mod exports;
pub mod start;
pub mod elements;
pub mod code;
pub mod data;

pub struct Module {
    pub version: u32,
    pub sections: HashMap<u64, Box<section::Section>>,
    // pub functions: Vec<functions::Function>,
}

impl Module {
    pub fn get_section<T: section::Section>(&self, index: u64) -> Option<&T> {
    	let value: &Box<section::Section> = self.sections.get(&index).unwrap();
    	value.downcast_ref::<T>()
    }
}
