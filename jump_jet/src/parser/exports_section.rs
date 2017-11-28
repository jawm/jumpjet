use std::collections::HashMap;
use std::io::Read;

use parser::leb::unsigned;
use parser::ParseError;

use tree::language_types::ExternalKind;
use tree::section::Section;
use tree::exports::ExportSection;
use tree::exports::ExportEntry;

pub fn parse(reader: &mut Read, sections: &HashMap<u64, Box<Section>>) -> Result<Box<Section>, ParseError> {
    let count = unsigned(&mut reader.bytes())?;
    let mut entries = vec![];
    for entry in 0..count {
        
        let field_len = unsigned(&mut reader.bytes())?;
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