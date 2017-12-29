use tree::language_types::ResizableLimits;

#[derive(Debug)]
pub struct Memory {
    pub limits: ResizableLimits,
    pub values: Vec<u8>
}