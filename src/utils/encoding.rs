// Encoding utilities

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

/// Decode text with specified encoding
pub fn decode_text(data: &[u8], encoding: TextEncoding) -> String {
    match encoding {
        TextEncoding::Iso8859_1 => {
            WINDOWS_1252.decode(data).0.to_string()
        }
        TextEncoding::Utf16 => {
            // Detect BOM
            if data.len() >= 2 {
                if &data[0..2] == [0xFF, 0xFE] {
                    UTF_16LE.decode(&data[2..]).0.to_string()
                } else if &data[0..2] == [0xFE, 0xFF] {
                    UTF_16BE.decode(&data[2..]).0.to_string()
                } else {
                    UTF_16LE.decode(data).0.to_string()
                }
            } else {
                String::new()
            }
        }
        TextEncoding::Utf16BE => {
            UTF_16BE.decode(data).0.to_string()
        }
        TextEncoding::Utf8 => {
            UTF_8.decode(data).0.to_string()
        }
    }
}

/// Encode text with specified encoding
#[allow(dead_code)]
pub fn encode_text(text: &str, encoding: TextEncoding) -> Vec<u8> {
    match encoding {
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
    }
}