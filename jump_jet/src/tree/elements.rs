pub struct ElementSection {
	entries: Vec<ElementSegment>
}

pub struct ElementSegment {
	index: u64,
	offset: i64, // TODO needs to be init_expr
	elements: Vec<u64>
}