

pub struct DataSection {
	entries: Vec<DataSegment>
}

pub struct DataSegment {
	index: u64,
	offset: i64, // TODO needs to be init_expr
	data: Vec<u8>
}