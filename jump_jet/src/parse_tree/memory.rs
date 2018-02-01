use parse_tree::language_types::ResizableLimits;

#[derive(Debug, Clone)]
pub struct Memory {
    pub limits: ResizableLimits,
    pub values: Vec<u8>
}