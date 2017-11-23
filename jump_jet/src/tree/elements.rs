use tree::section::Section;

#[derive(Debug)]
pub struct ElementSection {
    pub entries: Vec<ElementSegment>,
}

#[derive(Debug)]
pub struct ElementSegment {
    pub index: u64,
    pub offset: i64, // TODO needs to be init_expr
    pub elements: Vec<u64>,
}

impl Section for ElementSection {}
