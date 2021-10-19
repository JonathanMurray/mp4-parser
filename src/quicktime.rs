use crate::boxes::BoxHeader;
use crate::reader::Reader;

#[derive(Debug)]
pub struct MetadataItemList;

impl MetadataItemList {
    pub fn parse_entry(&self, reader: &mut Reader) -> EncoderTag {
        let header = BoxHeader::parse(reader);
        match header.box_type.as_ref() {
            "©too" => EncoderTag::parse(reader, header.inner_size),
            _ => todo!("Handle quicktime metadata item entry: {}", header.box_type),
        }
    }
}

#[derive(Debug)]
pub struct EncoderTag(String);

impl EncoderTag {
    pub fn parse(reader: &mut Reader, inner_size: u64) -> Self {
        let content = reader.read_string(inner_size as usize);
        Self(content)
    }
}
