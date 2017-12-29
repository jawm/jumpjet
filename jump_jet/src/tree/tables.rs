use tree::language_types::ResizableLimits;

#[derive(Debug)]
pub enum Table {
    AnyFunc {
        limits: ResizableLimits,
        values: Vec<usize>,
    }
}