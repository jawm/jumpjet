use tree::section::Section;

#[derive(Debug)]
pub struct DataSection {
    pub entries: Vec<DataSegment>,
}
#[derive(Debug)]
pub struct DataSegment {
    pub index: u64,
    pub offset: i64, // TODO needs to be init_expr
    pub data: Vec<u8>,
}

impl Section for DataSection {}
