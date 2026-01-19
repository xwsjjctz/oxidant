// VORBIS_COMMENT implementation for FLAC

use std::io::Read;

/// Vorbis comment structure
#[derive(Debug, Default)]
pub struct VorbisComment {
    pub vendor_string: String,
    pub comments: Vec<(String, String)>,
}

impl VorbisComment {
    /// Read Vorbis comment from reader
    pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        // Read vendor string length (little-endian 32-bit)
        let mut vendor_length_bytes = [0u8; 4];
        reader.read_exact(&mut vendor_length_bytes)?;
        let vendor_length = u32::from_le_bytes(vendor_length_bytes) as usize;

        // Read vendor string
        let mut vendor_bytes = vec![0u8; vendor_length];
        reader.read_exact(&mut vendor_bytes)?;
        let vendor_string = String::from_utf8_lossy(&vendor_bytes).to_string();

        // Read comment count (little-endian 32-bit)
        let mut comment_count_bytes = [0u8; 4];
        reader.read_exact(&mut comment_count_bytes)?;
        let comment_count = u32::from_le_bytes(comment_count_bytes) as usize;

        // Read comments
        let mut comments = Vec::with_capacity(comment_count);
        for _ in 0..comment_count {
            // Read comment length
            let mut comment_length_bytes = [0u8; 4];
            reader.read_exact(&mut comment_length_bytes)?;
            let comment_length = u32::from_le_bytes(comment_length_bytes) as usize;

            // Read comment string
            let mut comment_bytes = vec![0u8; comment_length];
            reader.read_exact(&mut comment_bytes)?;
            let comment_string = String::from_utf8_lossy(&comment_bytes).to_string();

            // Parse comment (format: FIELD=value)
            if let Some((field, value)) = comment_string.split_once('=') {
                comments.push((field.to_string(), value.to_string()));
            }
        }

        Ok(VorbisComment {
            vendor_string,
            comments,
        })
    }

    /// Get a comment value by field name
    pub fn get(&self, field: &str) -> Option<&String> {
        self.comments
            .iter()
            .find(|(f, _)| f.eq_ignore_ascii_case(field))
            .map(|(_, v)| v)
    }
}

/// Common Vorbis comment field names
pub struct VorbisFields;
impl VorbisFields {
    pub const TITLE: &str = "TITLE";
    pub const ARTIST: &str = "ARTIST";
    pub const ALBUM: &str = "ALBUM";
    pub const DATE: &str = "DATE";
    pub const TRACKNUMBER: &str = "TRACKNUMBER";
    pub const GENRE: &str = "GENRE";
    pub const COMMENT: &str = "COMMENT";
}

pub const VORBIS_FIELDS: VorbisFields = VorbisFields;