pub enum ValueType {
	i_32,
	i_64,
	f_32,
	f_64
}

pub enum LanguageType {
	i_32,
	i_64,
	f_32,
	f_64,
	anyfunc,
	func,
	empty_block	
}

pub enum ExternalKind {
	function(u64), // possibly have it go into the types section, instead of storing index
	table(TableType), 
	memory(ResizableLimits),
	global(GlobalType)
}

pub struct TableType {
	elemType: i64,
	limits: ResizableLimits
}

pub struct ResizableLimits {
	initial: u64,
	maximum: Option<u64>
}

pub struct GlobalType {
	contentType: ValueType,
	mutability: bool
}

pub enum Operation {

}