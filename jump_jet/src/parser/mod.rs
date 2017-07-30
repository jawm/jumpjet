
pub trait Parse {
	fn parse(reader: Read) -> Result<Section, ParseError>;
}

pub enum Error {

}