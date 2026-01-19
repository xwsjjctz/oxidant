// ID3v1 tag implementation

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

/// ID3v1 tag structure
#[derive(Debug, Default)]
pub struct Id3v1Tag {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub comment: String,
    pub track: Option<u8>,
    #[allow(dead_code)]
    pub genre: u8,
}

impl Id3v1Tag {
    const TAG_SIZE: usize = 128;
    const TAG_ID: [u8; 3] = [b'T', b'A', b'G'];

    /// Read ID3v1 tag from file
    pub fn read_from_file(path: &str) -> std::io::Result<Option<Self>> {
        let mut file = File::open(path)?;
        let file_size = file.metadata()?.len();

        if file_size < Self::TAG_SIZE as u64 {
            return Ok(None);
        }

        file.seek(SeekFrom::End(-(Self::TAG_SIZE as i64)))?;
        let mut buffer = [0u8; Self::TAG_SIZE];
        file.read_exact(&mut buffer)?;

        // Check for TAG identifier
        if &buffer[0..3] != Self::TAG_ID {
            return Ok(None);
        }

        Ok(Some(Self::parse(&buffer)))
    }

    /// Parse ID3v1 tag from buffer
    fn parse(buffer: &[u8; 128]) -> Self {
        let title = Self::parse_string(&buffer[3..33]);
        let artist = Self::parse_string(&buffer[33..63]);
        let album = Self::parse_string(&buffer[63..93]);
        let year = Self::parse_string(&buffer[93..97]);
        let comment = Self::parse_string(&buffer[97..127]);

        // Check for ID3v1.1 track number
        let (comment, track) = if buffer[125] == 0 && buffer[126] != 0 {
            (Self::parse_string(&buffer[97..125]), Some(buffer[126]))
        } else {
            (comment, None)
        };

        let genre = buffer[127];

        Id3v1Tag {
            title,
            artist,
            album,
            year,
            comment,
            track,
            genre,
        }
    }

    /// Parse null-terminated string
    fn parse_string(bytes: &[u8]) -> String {
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[..end]).trim().to_string()
    }
}