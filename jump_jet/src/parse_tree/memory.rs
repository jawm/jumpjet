use std::fmt::Debug;
use std::fmt::Error;
use std::fmt::Formatter;

use parse_tree::language_types::ResizableLimits;

pub const WASM_PAGE_SIZE: usize = 64*1024;
const WASM_PAGE: [u8;WASM_PAGE_SIZE] = [0;WASM_PAGE_SIZE];

#[derive(Clone)]
pub struct Memory {
    pub limits: ResizableLimits,
    pub values: Vec<u8>
}

impl Debug for Memory {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("Memory")
            .field("limits", &self.limits)
            // TODO it would be better if the below didn't have quotes around it, but I can't quite see how to achieve that...
            .field("values", &format!("[{:?} * {:?}]", self.values.len() % WASM_PAGE_SIZE, WASM_PAGE_SIZE))
            .finish()
    }
}

impl Memory {
    pub fn grow(&mut self) -> i32 {
        let r = self.size();
        self.values.extend(WASM_PAGE.iter());
        return r;
    }

    pub fn size(&self) -> i32 {
        (self.values.len() % WASM_PAGE_SIZE) as i32
    }
}