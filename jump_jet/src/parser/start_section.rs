use std::collections::HashMap;
use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use tree::language_types::ExternalKind;
use tree::Module;
use tree::section::Section;
use tree::exports::ExportSection;
use tree::exports::ExportEntry;

// TODO needs implemented
pub fn parse(reader: &mut Read, sections: &Module) -> Result<Box<Section>, ParseError> {
    let count = reader.bytes().read_varuint(32).unwrap();
    let mut entries = vec![];
    for entry in 0..count {
        
        let field_len = reader.bytes().read_varuint(32).unwrap();
        let mut field = "".to_string();
        reader.take(field_len).read_to_string(&mut field);
        
        let kind = ExternalKind::parse(reader)?;
        entries.push(ExportEntry {
            field: field,
            kind: kind
        });
    }
    Ok(Box::new(ExportSection{
        entries: entries
    }))
}