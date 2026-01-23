// OPUS format support (in OGG container)
//
// OPUS audio codec uses OGG container format with Vorbis Comment for metadata.
// This module handles OPUS-specific metadata while reusing the OGG infrastructure.
//
// OPUS File Structure:
// - Uses OGG container (similar to OGG Vorbis)
// - Identification header: "OpusHead" (8 bytes) in first page
// - Comment header: "OpusTags" (8 bytes) followed by Vorbis Comment in second page
// - Audio data pages
//
// Reference:
// - https://opus-codec.org/docs/
// - https://wiki.xiph.org/OggOpus
// - RFC 7845: Ogg Encapsulation for the Opus Audio Codec

use std::io::{BufRead, Read};
use std::fs::File;

#[allow(dead_code)]
pub const OPUS_SIGNATURE: &[u8; 8] = b"OpusHead";
#[allow(dead_code)]
pub const OPUS_TAGS: &[u8; 8] = b"OpusTags";

// Re-export FLAC's VorbisComment types since they're compatible
pub use crate::flac::vorbis::VorbisComment;

/// OPUS metadata handler
pub struct OpusFile {
    pub path: String,
}

impl OpusFile {
    /// Create a new OPUS file handler
    pub fn new(path: String) -> Self {
        OpusFile { path }
    }

    /// Read Vorbis comment from OPUS file
    pub fn read_comment(&self) -> std::io::Result<Option<VorbisComment>> {
        let file = File::open(&self.path)?;
        let mut reader = std::io::BufReader::new(file);

        // Try to read the OPUS comment page
        if let Some(comment_data) = read_opus_comment_page(&mut reader) {
            let mut cursor = std::io::Cursor::new(comment_data);
            return Ok(VorbisComment::read(&mut cursor).ok());
        }

        Ok(None)
    }

    /// Write Vorbis comment to OPUS file
    #[allow(dead_code)]
    pub fn write_comment(&self, comment: &VorbisComment) -> std::io::Result<()> {
        // Read the entire file
        let mut file_data = std::fs::read(&self.path)?;

        // Find and replace the comment page
        let mut pos = 0;
        let mut found_comment_page = false;

        while pos < file_data.len() {
            // Read page header
            if pos + 27 > file_data.len() {
                break;
            }

            // Check for OGG signature
            if &file_data[pos..pos + 4] != b"OggS" {
                break;
            }

            // Read segment count
            let segment_count = file_data[pos + 26] as usize;
            if pos + 27 + segment_count > file_data.len() {
                break;
            }

            // Read segment table
            let segment_table = &file_data[pos + 27..pos + 27 + segment_count];
            let data_size: usize = segment_table.iter().map(|&x| x as usize).sum();

            let header_size = 27 + segment_count;
            let total_page_size = header_size + data_size;

            // Check if this is page sequence 1 (comment page)
            let page_sequence = u32::from_le_bytes(file_data[pos + 18..pos + 22].try_into().unwrap());

            if page_sequence == 1 {
                // This is the comment page - replace it
                let new_comment_data = comment.to_bytes();

                // Construct new page data with OPUS comment header
                let mut new_page_data = Vec::new();
                new_page_data.extend_from_slice(OPUS_TAGS);
                new_page_data.extend_from_slice(&new_comment_data);

                // Update segment table for new data
                let new_data_size = new_page_data.len();
                let new_segment_table = create_segment_table(new_data_size);

                // Build new page
                let mut new_page = Vec::new();
                // Copy original header except segment table
                new_page.extend_from_slice(&file_data[pos..pos + 26]);
                // New segment count
                new_page.push(new_segment_table.len() as u8);
                // New segment table
                new_page.extend_from_slice(&new_segment_table);
                // New page data
                new_page.extend_from_slice(&new_page_data);

                // Replace page in file data
                let mut new_file_data = Vec::new();
                new_file_data.extend_from_slice(&file_data[..pos]);
                new_file_data.extend_from_slice(&new_page);
                new_file_data.extend_from_slice(&file_data[pos + total_page_size..]);

                file_data = new_file_data;
                found_comment_page = true;
                break;
            }

            pos += total_page_size;
        }

        if !found_comment_page {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "OPUS comment page not found"
            ));
        }

        // Write modified file
        std::fs::write(&self.path, file_data)?;

        Ok(())
    }
}

/// Read OPUS comment page from reader
fn read_opus_comment_page<R: BufRead>(reader: &mut R) -> Option<Vec<u8>> {
    loop {
        // Read page header
        let mut header = [0u8; 27];
        if reader.read_exact(&mut header).is_err() {
            return None;
        }

        // Check OGG signature
        if &header[0..4] != b"OggS" {
            return None;
        }

        // Read segment count
        let segment_count = header[26];
        let mut segment_table = vec![0u8; segment_count as usize];
        if reader.read_exact(&mut segment_table).is_err() {
            return None;
        }

        // Calculate data size
        let data_size: usize = segment_table.iter().map(|&x| x as usize).sum();

        // Check page sequence
        let page_sequence = u32::from_le_bytes(header[18..22].try_into().unwrap());

        if page_sequence == 1 {
            // This is the comment header page
            // Read page data
            let mut data = vec![0u8; data_size];
            if reader.read_exact(&mut data).is_err() {
                return None;
            }

            // Data starts with "OpusTags" (8 bytes), skip it and return comment data
            if data.len() > 8 && &data[0..8] == OPUS_TAGS {
                return Some(data[8..].to_vec());
            }
        } else {
            // Skip the data
            let mut skip_buf = vec![0u8; data_size.min(8192)];
            let mut remaining = data_size;
            while remaining > 0 {
                let to_read = remaining.min(skip_buf.len());
                if reader.read_exact(&mut skip_buf[0..to_read]).is_err() {
                    return None;
                }
                remaining -= to_read;
            }

            // Stop if we've passed the comment page
            if page_sequence > 1 {
                break;
            }
        }
    }
    None
}

/// Create segment table for given data size
#[allow(dead_code)]
fn create_segment_table(size: usize) -> Vec<u8> {
    let mut table = Vec::new();
    let mut remaining = size;

    while remaining > 0 {
        let segment_size = remaining.min(255);
        table.push(segment_size as u8);
        remaining -= segment_size;
    }

    table
}

/// Detect if file is OPUS format
#[allow(dead_code)]
pub fn is_opus_file(path: &str) -> bool {
    if let Ok(mut file) = File::open(path) {
        let mut signature = [0u8; 4];
        if file.read_exact(&mut signature).is_ok() {
            // Check for OGG container
            if &signature == b"OggS" {
                // Check if first page contains OpusHead
                let mut page_header = [0u8; 27];
                if file.read_exact(&mut page_header).is_err() {
                    return false;
                }

                let segment_count = page_header[26] as usize;
                let mut segment_table = vec![0u8; segment_count];
                if file.read_exact(&mut segment_table).is_ok() {
                    let data_size: usize = segment_table.iter().map(|&x| x as usize).sum();
                    if data_size >= 8 {
                        let mut opus_sig = [0u8; 8];
                        if file.read_exact(&mut opus_sig).is_ok() {
                            return &opus_sig == OPUS_SIGNATURE;
                        }
                    }
                }
            }
        }
    }
    false
}
