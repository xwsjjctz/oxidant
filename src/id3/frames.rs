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
#[allow(dead_code)]
pub mod frame_ids {
    pub const TITLE: &str = "TIT2";  // Title/songname/content description
    pub const ARTIST: &str = "TPE1"; // Lead performer(s)/Soloist(s)
    pub const ALBUM: &str = "TALB";  // Album/Movie/Show title
    pub const YEAR: &str = "TYER";   // Year
    pub const TRACK: &str = "TRCK";  // Track number/Position in set
    pub const GENRE: &str = "TCON";  // Content type
    pub const COMMENT: &str = "COMM"; // Comments
    pub const PICTURE: &str = "APIC"; // Attached picture
    pub const LYRICS: &str = "USLT"; // Unsynchronized lyrics
}

/// Decode text frame data
#[allow(dead_code)]
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
#[allow(dead_code)]
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

/// Picture type for ID3v2 APIC frame
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PictureType {
    Other = 0x00,
    FileIcon = 0x01,
    OtherFileIcon = 0x02,
    CoverFront = 0x03,
    CoverBack = 0x04,
    LeafletPage = 0x05,
    Media = 0x06,
    LeadArtist = 0x07,
    Artist = 0x08,
    Conductor = 0x09,
    Band = 0x0A,
    Composer = 0x0B,
    Lyricist = 0x0C,
    RecordingLocation = 0x0D,
    DuringRecording = 0x0E,
    DuringPerformance = 0x0F,
    VideoScreenCapture = 0x10,
    BrightColouredFish = 0x11,
    Illustration = 0x12,
    BandLogo = 0x13,
    PublisherLogo = 0x14,
}

impl PictureType {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => PictureType::Other,
            0x01 => PictureType::FileIcon,
            0x02 => PictureType::OtherFileIcon,
            0x03 => PictureType::CoverFront,
            0x04 => PictureType::CoverBack,
            0x05 => PictureType::LeafletPage,
            0x06 => PictureType::Media,
            0x07 => PictureType::LeadArtist,
            0x08 => PictureType::Artist,
            0x09 => PictureType::Conductor,
            0x0A => PictureType::Band,
            0x0B => PictureType::Composer,
            0x0C => PictureType::Lyricist,
            0x0D => PictureType::RecordingLocation,
            0x0E => PictureType::DuringRecording,
            0x0F => PictureType::DuringPerformance,
            0x10 => PictureType::VideoScreenCapture,
            0x11 => PictureType::BrightColouredFish,
            0x12 => PictureType::Illustration,
            0x13 => PictureType::BandLogo,
            0x14 => PictureType::PublisherLogo,
            _ => PictureType::Other,
        }
    }
}

/// Encode APIC (Attached Picture) frame
pub fn encode_apic_frame(
    mime_type: &str,
    picture_type: PictureType,
    description: &str,
    image_data: &[u8],
) -> Vec<u8> {
    let mut result = Vec::new();

    // Text encoding (use ISO-8859-1 for MIME type and description)
    result.push(TextEncoding::Iso8859_1 as u8);

    // MIME type (null-terminated)
    result.extend_from_slice(mime_type.as_bytes());
    result.push(0);

    // Picture type
    result.push(picture_type as u8);

    // Description (null-terminated, ISO-8859-1)
    result.extend_from_slice(WINDOWS_1252.encode(description).0.as_ref());
    result.push(0);

    // Image data
    result.extend_from_slice(image_data);

    result
}

/// Decode APIC (Attached Picture) frame
pub fn decode_apic_frame(data: &[u8]) -> Option<(String, PictureType, String, Vec<u8>)> {
    if data.is_empty() {
        return None;
    }

    let pos = 0;

    // Text encoding
    let encoding = TextEncoding::from_byte(data[pos]);

    // Find MIME type (null-terminated)
    let mut mime_end = pos + 1;
    while mime_end < data.len() && data[mime_end] != 0 {
        mime_end += 1;
    }
    if mime_end >= data.len() {
        return None;
    }
    let mime_type = String::from_utf8_lossy(&data[pos + 1..mime_end]).to_string();

    // Picture type
    let picture_type = PictureType::from_byte(data[mime_end + 1]);

    // Find description (null-terminated)
    let desc_start = mime_end + 2;
    let mut desc_end = desc_start;
    while desc_end < data.len() && data[desc_end] != 0 {
        desc_end += 1;
    }
    if desc_end >= data.len() {
        return None;
    }
    
    // Decode description based on encoding
    let description = if desc_end > desc_start {
        decode_text_frame_with_encoding(&data[desc_start..desc_end], encoding)
    } else {
        String::new()
    };

    // Image data
    let image_data = data[desc_end + 1..].to_vec();

    Some((mime_type, picture_type, description, image_data))
}

/// Decode text with specific encoding
fn decode_text_frame_with_encoding(data: &[u8], encoding: TextEncoding) -> String {
    if data.is_empty() {
        return String::new();
    }

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

/// Encode USLT (Unsynchronized Lyrics) frame
pub fn encode_uslt_frame(
    language: &str,
    description: &str,
    lyrics: &str,
) -> Vec<u8> {
    let mut result = Vec::new();

    // Text encoding (use UTF-8 for better multilingual support)
    result.push(TextEncoding::Utf8 as u8);

    // Language (3 bytes, ISO-639-2)
    let lang_bytes = language.as_bytes();
    if lang_bytes.len() >= 3 {
        result.extend_from_slice(&lang_bytes[0..3]);
    } else {
        result.extend_from_slice(lang_bytes);
        result.extend_from_slice(&vec![0u8; 3 - lang_bytes.len()]);
    }

    // Description (null-terminated)
    result.extend_from_slice(UTF_8.encode(description).0.as_ref());
    result.push(0);

    // Lyrics text
    result.extend_from_slice(UTF_8.encode(lyrics).0.as_ref());

    result
}

/// Decode USLT (Unsynchronized Lyrics) frame
pub fn decode_uslt_frame(data: &[u8]) -> Option<(String, String, String)> {
    if data.is_empty() {
        return None;
    }

    // Text encoding
    let encoding = TextEncoding::from_byte(data[0]);

    // Language (3 bytes)
    if data.len() < 4 {
        return None;
    }
    let language = String::from_utf8_lossy(&data[1..4]).to_string();

    // Find description (null-terminated)
    let desc_start = 4;
    let mut desc_end = desc_start;
    while desc_end < data.len() && data[desc_end] != 0 {
        desc_end += 1;
    }
    if desc_end >= data.len() {
        return None;
    }

    // Decode description based on encoding
    let description = if desc_end > desc_start {
        decode_text_frame_with_encoding(&data[desc_start..desc_end], encoding)
    } else {
        String::new()
    };

    // Lyrics (remaining data after null terminator)
    let lyrics_start = desc_end + 1;
    let lyrics = if lyrics_start < data.len() {
        decode_text_frame_with_encoding(&data[lyrics_start..], encoding)
    } else {
        String::new()
    };

    Some((language, description, lyrics))
}