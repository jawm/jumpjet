use mopa::Any;

use std::fmt::Debug;

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



pub trait Section: Any {}
mopafy!(Section);