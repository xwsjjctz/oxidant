// OGG Vorbis Comment implementation
// Reuses the FLAC Vorbis Comment structure since the format is identical

use std::io::{Read, BufReader};
use std::fs::File;

// Re-export FLAC's Vorbis Comment types since they're compatible
pub use crate::flac::vorbis::VorbisComment;

/// OGG Vorbis metadata reader/writer
pub struct OggVorbisFile {
    pub path: String,
}

impl OggVorbisFile {
    /// Create a new OGG Vorbis file handler
    pub fn new(path: String) -> Self {
        OggVorbisFile { path }
    }

    /// Read Vorbis comment from OGG file
    pub fn read_comment(&self) -> std::io::Result<Option<VorbisComment>> {
        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);

        // Try to read the Vorbis comment page
        if let Some(comment_data) = crate::ogg::page::OggPage::read_vorbis_comment_page(&mut reader) {
            let mut cursor = std::io::Cursor::new(comment_data);
            return Ok(VorbisComment::read(&mut cursor).ok());
        }

        Ok(None)
    }

    /// Write Vorbis comment to OGG file
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

                // Construct new page data with Vorbis comment header
                let mut new_page_data = Vec::new();
                new_page_data.push(0x03); // Packet type (comment header)
                new_page_data.extend_from_slice(b"vorbis");
                new_page_data.extend_from_slice(&new_comment_data);

                // Update segment table for new data
                let new_data_size = new_page_data.len();
                let new_segment_table = Self::create_segment_table(new_data_size);

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
                "Vorbis comment page not found"
            ));
        }

        // Write modified file
        std::fs::write(&self.path, file_data)?;

        Ok(())
    }

    /// Create segment table for given data size
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
}

/// Detect if file is OGG format
#[allow(dead_code)]
pub fn is_ogg_file(path: &str) -> bool {
    if let Ok(mut file) = File::open(path) {
        let mut signature = [0u8; 4];
        if file.read_exact(&mut signature).is_ok() {
            return &signature == b"OggS";
        }
    }
    false
}
