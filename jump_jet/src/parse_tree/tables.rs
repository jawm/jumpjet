use parse_tree::language_types::ResizableLimits;

#[derive(Debug, Clone)]
pub enum Table {
    AnyFunc {
        limits: ResizableLimits,
        values: Vec<usize>,
    }
}