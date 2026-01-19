// FLAC metadata block implementation

use std::io::Read;

/// FLAC metadata block types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlacMetadataBlockType {
    StreamInfo = 0,
    Padding = 1,
    Application = 2,
    SeekTable = 3,
    VorbisComment = 4,
    CueSheet = 5,
    Picture = 6,
    Invalid = 127,
}

impl FlacMetadataBlockType {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0 => FlacMetadataBlockType::StreamInfo,
            1 => FlacMetadataBlockType::Padding,
            2 => FlacMetadataBlockType::Application,
            3 => FlacMetadataBlockType::SeekTable,
            4 => FlacMetadataBlockType::VorbisComment,
            5 => FlacMetadataBlockType::CueSheet,
            6 => FlacMetadataBlockType::Picture,
            127 => FlacMetadataBlockType::Invalid,
            _ => FlacMetadataBlockType::Invalid,
        }
    }
}

/// FLAC metadata block header
#[derive(Debug)]
pub struct FlacMetadataBlockHeader {
    pub is_last: bool,
    pub block_type: FlacMetadataBlockType,
    pub length: u32,
}

/// FLAC metadata block
#[derive(Debug)]
pub struct FlacMetadataBlock {
    pub header: FlacMetadataBlockHeader,
    pub data: Vec<u8>,
}

impl FlacMetadataBlockHeader {
    const HEADER_SIZE: usize = 4;

    /// Read FLAC metadata block header from reader
    pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buffer = [0u8; Self::HEADER_SIZE];
        reader.read_exact(&mut buffer)?;

        let is_last = (buffer[0] & 0x80) != 0;
        let block_type = FlacMetadataBlockType::from_byte(buffer[0] & 0x7F);

        // Length is big-endian 24-bit
        let length = ((buffer[1] as u32) << 16) |
                    ((buffer[2] as u32) << 8) |
                    (buffer[3] as u32);

        Ok(FlacMetadataBlockHeader {
            is_last,
            block_type,
            length,
        })
    }
}

impl FlacMetadataBlock {
    /// Read FLAC metadata block from reader
    pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let header = FlacMetadataBlockHeader::read(reader)?;
        let mut data = vec![0u8; header.length as usize];
        reader.read_exact(&mut data)?;

        Ok(FlacMetadataBlock { header, data })
    }
}

/// FLAC file signature
pub const FLAC_SIGNATURE: &[u8; 4] = b"fLaC";