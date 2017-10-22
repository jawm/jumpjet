use super::types::TypeSection;
use super::imports::ImportSection;
use super::functions::FunctionSection;
use super::tables::TableSection;
use super::memory::MemorySection;
use super::globals::GlobalSection;
use super::exports::ExportSection;
use super::start::StartSection;
use super::elements::ElementSection;
use super::code::CodeSection;
use super::data::DataSection;

// pub enum Section {
// 	Type(TypeSection),
// 	Import(ImportSection),
// 	Function(FunctionSection),
// 	Table(TableSection),
// 	Memory(MemorySection),
// 	Global(GlobalSection),
// 	Export(ExportSection),
// 	Start(StartSection),
// 	Element(ElementSection),
// 	Code(CodeSection),
// 	Data(DataSection)
// }

pub trait Section {
	
}