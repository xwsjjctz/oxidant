use std::io::{Read, BufRead};
use crate::ogg::{OGG_SIGNATURE, OGG_HEADER_TYPE_BOS};

/// OGG Page Header
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OggPageHeader {
    pub(crate) version: u8,
    pub(crate) header_type: u8,
    pub(crate) granule_position: u64,
    pub(crate) bitstream_serial: u32,
    pub page_sequence: u32,
    pub(crate) crc: u32,
    pub(crate) segment_count: u8,
    pub segment_table: Vec<u8>,
}

/// OGG Page
#[derive(Debug, Clone)]
pub struct OggPage {
    pub header: OggPageHeader,
    pub data: Vec<u8>,
}

impl OggPageHeader {
    /// Read OGG page header from a reader
    pub fn read<R: Read>(reader: &mut R) -> Option<Self> {
        let mut header = [0u8; 27];
        if reader.read_exact(&mut header).is_err() {
            return None;
        }

        // Check OGG signature
        if &header[0..4] != OGG_SIGNATURE {
            return None;
        }

        let version = header[4];
        if version != 0 {
            return None;
        }

        let header_type = header[5];
        let granule_position = u64::from_le_bytes(header[6..14].try_into().unwrap());
        let bitstream_serial = u32::from_le_bytes(header[14..18].try_into().unwrap());
        let page_sequence = u32::from_le_bytes(header[18..22].try_into().unwrap());
        let crc = u32::from_le_bytes(header[22..26].try_into().unwrap());
        let segment_count = header[26];

        // Read segment table
        let mut segment_table = vec![0u8; segment_count as usize];
        if reader.read_exact(&mut segment_table).is_err() {
            return None;
        }

        Some(OggPageHeader {
            version,
            header_type,
            granule_position,
            bitstream_serial,
            page_sequence,
            crc,
            segment_count,
            segment_table,
        })
    }

    /// Calculate total page data size from segment table
    pub fn get_data_size(&self) -> usize {
        self.segment_table.iter().map(|&x| x as usize).sum()
    }

    /// Check if this is the beginning of a stream
    #[allow(dead_code)]
    pub(crate) fn is_bos(&self) -> bool {
        self.header_type & OGG_HEADER_TYPE_BOS != 0
    }
}

impl OggPage {
    /// Read OGG page from a reader
    pub fn read<R: Read>(reader: &mut R) -> Option<Self> {
        let header = OggPageHeader::read(reader)?;

        // Read page data
        let data_size = header.get_data_size();
        let mut data = vec![0u8; data_size];
        if reader.read_exact(&mut data).is_err() {
            return None;
        }

        Some(OggPage { header, data })
    }

    /// Read page and check if it contains Vorbis comment
    /// Vorbis comment is in the second page (page_sequence == 1)
    pub fn read_vorbis_comment_page<R: BufRead>(reader: &mut R) -> Option<Vec<u8>> {
        loop {
            let page = Self::read(reader)?;
            if page.header.page_sequence == 1 {
                // This is the comment header page
                // Data starts with packet type (0x03) and "vorbis" identifier
                if page.data.len() > 7 && page.data[0] == 0x03 && &page.data[1..7] == b"vorbis" {
                    // Skip the header and return comment data
                    return Some(page.data[7..].to_vec());
                }
            }
            // Stop if we've passed the comment page
            if page.header.page_sequence > 1 {
                break;
            }
        }
        None
    }
}
