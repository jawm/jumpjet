use std::collections::HashMap;
use std::io::Read;

use parser::byteorder::ReadBytesExt;
use parser::leb::unsigned;
use parser::leb::signed;
use parser::ParseError;

use tree::section::Section;
use tree::data::DataSection;
use tree::data::DataSegment;

// TODO finish implementing.
pub fn parse(reader: &mut Read, sections: &HashMap<u64, Box<Section>>) -> Result<Box<Section>, ParseError> {
    println!("Got this far?!");
    let count = unsigned(&mut reader.bytes())?;
    let mut entries = vec![];
    println!("count: {}", count);
    for entry in 0..count {
        println!("iteration");
        let index = unsigned(&mut reader.bytes())?;
        let offset = signed(&mut reader.bytes())?;
        let size = unsigned(&mut reader.bytes())?;
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