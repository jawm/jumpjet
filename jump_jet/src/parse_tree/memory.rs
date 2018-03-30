use parse_tree::language_types::ResizableLimits;

pub const WASM_PAGE_SIZE: usize = 64*1024;
const WASM_PAGE: [u8;WASM_PAGE_SIZE] = [0;WASM_PAGE_SIZE];

#[derive(Debug, Clone)]
pub struct Memory {
    pub limits: ResizableLimits,
    pub values: Vec<u8>
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