use std::collections::HashMap;
use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use tree::Module;
use tree::section::Section;
use tree::data::DataSection;
use tree::data::DataSegment;

// TODO finish implementing.
pub fn parse(reader: &mut Read, sections: &Module) -> Result<Box<Section>, ParseError> {
    println!("Got this far?!");
    let count = reader.bytes().read_varuint(32).unwrap();
    let mut entries = vec![];
    println!("count: {}", count);
    for entry in 0..count {
        println!("iteration");
        let index = reader.bytes().read_varuint(32).unwrap();
        let offset = 0; // TODO FIGURE OUT PARSING OF EXPRESSIONS - SHOULD BE I32 INITIALISER
        let size = reader.bytes().read_varuint(32).unwrap();
        let mut data = vec![];
        reader.take(size).read_to_end(&mut data);
        entries.push(DataSegment {
            index: index,
            offset: offset,
            data: data
        });
    }
    println!("returning");
    Ok(Box::new(DataSection{
        entries: entries
    }))
}