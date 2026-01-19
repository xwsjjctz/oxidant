// ID3 frame definitions and parsers

use encoding_rs::{UTF_16BE, UTF_16LE, UTF_8, WINDOWS_1252};

/// Text encoding types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextEncoding {
    Iso8859_1 = 0,
    Utf16 = 1,
    Utf16BE = 2,
    Utf8 = 3,
}

impl TextEncoding {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0 => TextEncoding::Iso8859_1,
            1 => TextEncoding::Utf16,
            2 => TextEncoding::Utf16BE,
            3 => TextEncoding::Utf8,
            _ => TextEncoding::Iso8859_1,
        }
    }
}

/// Common ID3v2.3 frame identifiers
pub mod frame_ids {
    pub const TITLE: &str = "TIT2";  // Title/songname/content description
    pub const ARTIST: &str = "TPE1"; // Lead performer(s)/Soloist(s)
    pub const ALBUM: &str = "TALB";  // Album/Movie/Show title
    pub const YEAR: &str = "TYER";   // Year
    pub const TRACK: &str = "TRCK";  // Track number/Position in set
    pub const GENRE: &str = "TCON";  // Content type
    pub const COMMENT: &str = "COMM"; // Comments
    pub const PICTURE: &str = "APIC"; // Attached picture
}

/// Decode text frame data
pub fn decode_text_frame(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let encoding = TextEncoding::from_byte(data[0]);
    let text_data = &data[1..];

    match encoding {
        TextEncoding::Iso8859_1 => {
            WINDOWS_1252.decode(text_data).0.to_string()
        }
        TextEncoding::Utf16 => {
            // Detect BOM
            if text_data.len() >= 2 {
                if &text_data[0..2] == [0xFF, 0xFE] {
                    UTF_16LE.decode(&text_data[2..]).0.to_string()
                } else if &text_data[0..2] == [0xFE, 0xFF] {
                    UTF_16BE.decode(&text_data[2..]).0.to_string()
                } else {
                    UTF_16LE.decode(text_data).0.to_string()
                }
            } else {
                String::new()
            }
        }
        TextEncoding::Utf16BE => {
            UTF_16BE.decode(text_data).0.to_string()
        }
        TextEncoding::Utf8 => {
            UTF_8.decode(text_data).0.to_string()
        }
    }
}

/// Encode text frame data
pub fn encode_text_frame(text: &str, encoding: TextEncoding) -> Vec<u8> {
    let mut result = vec![encoding as u8];

    let encoded = match encoding {
        TextEncoding::Iso8859_1 => {
            WINDOWS_1252.encode(text).0.to_vec()
        }
        TextEncoding::Utf16 => {
            let mut bom = vec![0xFF, 0xFE];
            let encoded = UTF_16LE.encode(text).0.to_vec();
            bom.extend(encoded);
            bom
        }
        TextEncoding::Utf16BE => {
            let mut bom = vec![0xFE, 0xFF];
            let encoded = UTF_16BE.encode(text).0.to_vec();
            bom.extend(encoded);
            bom
        }
        TextEncoding::Utf8 => {
            UTF_8.encode(text).0.to_vec()
        }
    };

    result.extend(encoded);
    result
}