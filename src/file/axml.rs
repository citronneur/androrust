extern crate byteorder;

use std::io::{Read, Error};
use self::byteorder::{LittleEndian, ReadBytesExt};

const ATTRIBUTE_LENGHT: u32 = 5;

const CHUNK_NULL: i32 = 0x00000000;
const CHUNK_STRINGPOOL_TYPE: i32 = 0x001C0001;

const CHUNK_AXML_FILE: u32 = 0x00080003;
const CHUNK_RESSOURCEIDS: u32 = 0x00080180;
const CHUNK_XML_START_TAG: u32 = 0x00100102;
const CHUNK_XML_START_NAMESPACE: u32 = 0x00100100;
const CHUNK_XML_END_NAMESPACE: u32 = 0x00100101;
const CHUNK_XML_END_TAG: u32 = 0x00100103;
const CHUNK_XML_TEXT: u32 = 0x00100104;
const CHUNK_XML_LAST: u32 = 0x00100104;

#[derive(Debug)]
pub enum AxmlError {
    IoError(Error),
    InvalidStartElement,
    InvalidStringBlockHeader,
    InvalidStringBlockSize,
    InvalidChunkSize,
    InvalidTag
}

impl From<Error> for AxmlError {
    fn from(e: Error) -> AxmlError{
        AxmlError::IoError(e)
    }
}

enum Chunk {
    StartTag(Tag, Attribute),
    EndTag(Tag),
    StartNamespace(Namespace),
    EndNamespace(Namespace),
    Text(Text),
    Resources(Vec<u32>)
}

struct Header {
    size: u32,
    line: u32,
}

struct Tag {
    header: Header,
    namespace_uri: u32,
    name: u32
}

struct Attribute {
    id_attribute: u16,
    class_attribute: u16,
    style_attribute: u16,
    attributes: Vec<u32>
}

struct Namespace {
    header: Header,
    prefix: u32,
    uri: u32
}

struct Text {
    header: Header,
    name: u32,
    unknown1: u32,
    unknown2: u32
}

type AxmlResult<T> = Result<T, AxmlError>;

pub struct Axml {
    string_block: StringBlock,
    chunks: Vec<Chunk>
}

fn read_header(buffer: &mut Read) -> Result<Header, AxmlError> {
    let size = buffer.read_u32::<LittleEndian>()?;
    let line = buffer.read_u32::<LittleEndian>()?;
    buffer.read_u32::<LittleEndian>()?; // padding 0xFFFFFFFF
    Ok(Header {
        size: size,
        line: line
    })
}

impl Axml {
    pub fn read(buffer: &mut Read) -> Result<Axml, AxmlError> {
        // try to read the file
        if buffer.read_u32::<LittleEndian>()? != CHUNK_AXML_FILE {
            return Err(AxmlError::InvalidStartElement)
        }

        // read padding
        buffer.read_u32::<LittleEndian>()?;

        let string_blocks = StringBlock::read(buffer)?;
        let mut chunks = Vec::<Chunk>::new();

        loop {
            chunks.push(
                match buffer.read_u32::<LittleEndian>()? {
                    CHUNK_RESSOURCEIDS => {
                        let mut ressource_ids = Vec::<u32>::new();
                        let chunk_size = buffer.read_u32::<LittleEndian>()?;
                        if chunk_size < 8 || chunk_size % 4 != 0 {
                            return Err(AxmlError::InvalidChunkSize);
                        }
                        for i in 0..(chunk_size / 4 - 2) {
                            ressource_ids.push(buffer.read_u32::<LittleEndian>()?);
                        }
                        Chunk::Resources(ressource_ids)
                    },
                    CHUNK_XML_START_NAMESPACE => {
                        Chunk::StartNamespace(Namespace {
                            header: read_header(buffer)?,
                            prefix: buffer.read_u32::<LittleEndian>()?,
                            uri: buffer.read_u32::<LittleEndian>()?
                        })
                    },
                    CHUNK_XML_END_NAMESPACE => {
                        Chunk::EndNamespace(Namespace {
                            header: read_header(buffer)?,
                            prefix: buffer.read_u32::<LittleEndian>()?,
                            uri: buffer.read_u32::<LittleEndian>()?
                        })
                    },
                    CHUNK_XML_START_TAG => {
                        let header = read_header(buffer)?;
                        let namespace_uri = buffer.read_u32::<LittleEndian>()?;
                        let name = buffer.read_u32::<LittleEndian>()?;
                        let flags = buffer.read_u32::<LittleEndian>()?;
                        let attribute_id_count = buffer.read_u32::<LittleEndian>()?;
                        let id_attribute = (attribute_id_count >> 16) as u16;
                        let attribute_count = attribute_id_count as u16;
                        let class_and_style = buffer.read_u32::<LittleEndian>()?;
                        let class_attribute = class_and_style as u16;
                        let style_attribute = (class_and_style >> 16) as u16;
                        let mut attributes = Vec::new();

                        for i in 0..(attribute_count as u32 * ATTRIBUTE_LENGHT) {
                            attributes.push(buffer.read_u32::<LittleEndian>()?)
                        }

                        Chunk::StartTag(Tag {
                            header: header,
                            namespace_uri: namespace_uri,
                            name: name,
                        }, Attribute {
                            id_attribute: id_attribute,
                            class_attribute: class_attribute,
                            style_attribute: style_attribute,
                            attributes: attributes
                        })
                    },
                    CHUNK_XML_END_TAG => {

                        Chunk::EndTag(Tag {
                            header: read_header(buffer)?,
                            namespace_uri: buffer.read_u32::<LittleEndian>()?,
                            name: buffer.read_u32::<LittleEndian>()?
                        })
                    },
                    CHUNK_XML_TEXT => {
                        Chunk::Text(Text {
                            header: read_header(buffer)?,
                            name: buffer.read_u32::<LittleEndian>()?,
                            unknown1: buffer.read_u32::<LittleEndian>()?,
                            unknown2: buffer.read_u32::<LittleEndian>()?
                        })
                    },
                    _ => { return Err(AxmlError::InvalidTag); }
                }
            )
        }

        Ok(Axml {
            string_block: string_blocks,
            chunks: chunks
        })
    }
}

struct StringBlock {
    chunk_size: i32,
    string_count: i32,
    style_offset_count: i32,
    flags: i32,
    strings_offset: i32,
    styles_offset: i32,
    string_offsets: Vec<i32>,
    style_offsets: Vec<i32>,
    styles: Vec<i32>,
    char_buffer: Vec<u8>
}

impl StringBlock {

    // Read
    fn read(buffer: &mut Read) -> Result<StringBlock, AxmlError>{
        // read header
        loop {
            match buffer.read_i32::<LittleEndian>()? {
                CHUNK_NULL => continue,
                CHUNK_STRINGPOOL_TYPE => break,
                _ => return Err(AxmlError::InvalidStringBlockHeader)
            };
        }

        let chunk_size = buffer.read_i32::<LittleEndian>()?;
        let string_count = buffer.read_i32::<LittleEndian>()?;
        let style_offset_count = buffer.read_i32::<LittleEndian>()?;
        let flags = buffer.read_i32::<LittleEndian>()?;
        let strings_offset = buffer.read_i32::<LittleEndian>()?;
        let mut styles_offset = buffer.read_i32::<LittleEndian>()?;

        let mut string_offsets = Vec::<i32>::new();

        for i in 0..string_count {
            string_offsets.push(buffer.read_i32::<LittleEndian>()?);
        }

        let mut style_offsets = Vec::<i32>::new();

        for i in 0..style_offset_count {
            style_offsets.push(buffer.read_i32::<LittleEndian>()?);
        }

        if styles_offset == 0 {
            styles_offset = chunk_size;
        }

        let mut char_buffer_size = styles_offset - strings_offset;

        if char_buffer_size % 4 != 0 {
            return Err(AxmlError::InvalidStringBlockSize);
        }

        let mut char_buffer = vec![0; char_buffer_size as usize];
        buffer.read(&mut char_buffer);

        let styles_size = chunk_size - styles_offset;
        let mut styles = Vec::<i32>::new();
        for i in 0..(styles_size / 4) {
            styles.push(buffer.read_i32::<LittleEndian>()?);
        }

        Ok(StringBlock {
            chunk_size: chunk_size,
            string_count: string_count,
            style_offset_count: style_offset_count,
            flags: flags,
            strings_offset: strings_offset,
            styles_offset: styles_offset,
            string_offsets: string_offsets,
            style_offsets: style_offsets,
            char_buffer: char_buffer,
            styles: styles
        })
    }
}
