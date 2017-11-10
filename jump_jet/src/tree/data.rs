use tree::section::Section;

pub struct DataSection {
	pub entries: Vec<DataSegment>
}

pub struct DataSegment {
	pub index: u64,
	pub offset: i64, // TODO needs to be init_expr
	pub data: Vec<u8>
}

impl Section for DataSection {}