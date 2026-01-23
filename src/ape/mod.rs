// APE format support
//
// APE (Monkey's Audio) is a lossless audio codec.
// It uses APE tags for metadata storage (similar to ID3 but different format).
//
// APE Tag Structure:
// - Tag header/footer (32 bytes):
//   - Signature: "APETAGEX" (8 bytes)
//   - Version: 2000 (4 bytes, little-endian)
//   - Tag size: (4 bytes)
//   - Item count: (4 bytes)
//   - Flags: (4 bytes)
//   - Reserved: (8 bytes)
// - Tag items (variable count, at end of file)
//   - Item size: (4 bytes, little-endian)
//   - Item flags: (4 bytes)
//   - Key: UTF-8 string (null-terminated)
//   - Value: UTF-8 string (variable size)
//
// APE tags are typically at the END of the file (footer + items, optional header at beginning)
//
// Common APE tag fields:
// - Title: Title
// - Artist: Artist
// - Album: Album
// - Year: Year
// - Track: Track
// - Genre: Genre
// - Comment: Comment
// - Lyrics: Lyrics

pub const APE_SIGNATURE: &[u8; 8] = b"APETAGEX";
pub const APE_VERSION: u32 = 2000;

// APE tag field names
pub mod fields {
    pub const TITLE: &str = "Title";
    pub const ARTIST: &str = "Artist";
    pub const ALBUM: &str = "Album";
    pub const YEAR: &str = "Year";
    pub const TRACK: &str = "Track";
    pub const GENRE: &str = "Genre";
    pub const COMMENT: &str = "Comment";
    pub const LYRICS: &str = "Lyrics";
}

// APE tag flags
#[allow(dead_code)]
pub mod flags {
    #[allow(dead_code)]
    pub const CONTAINS_HEADER: u32 = 0x80000000;
    #[allow(dead_code)]
    pub const CONTAINS_FOOTER: u32 = 0x40000000;
    pub const IS_HEADER: u32 = 0x20000000;
    #[allow(dead_code)]
    pub const READ_ONLY: u32 = 0x10000000;
}

/// APE tag header/footer
#[derive(Debug, Clone)]
pub struct ApeTagHeader {
    #[allow(dead_code)]
    pub version: u32,
    pub tag_size: u32,
    pub item_count: u32,
    pub flags: u32,
    #[allow(dead_code)]
    pub reserved: [u8; 8],
}

/// APE tag item
#[derive(Debug, Clone)]
pub struct ApeTagItem {
    pub size: u32,
    #[allow(dead_code)]
    pub flags: u32,
    pub key: String,
    pub value: Vec<u8>,
}

/// APE metadata handler
pub struct ApeFile {
    pub path: String,
}

impl ApeFile {
    /// Create a new APE file handler
    pub fn new(path: String) -> Self {
        ApeFile { path }
    }

    /// Read metadata from APE file
    pub fn read_metadata(&self) -> std::io::Result<Option<ApeMetadata>> {
        let file_data = std::fs::read(&self.path)?;

        // APE tags are at the end of the file
        // Try to find the APE tag footer
        if let Some((_header, items)) = self.parse_ape_tag(&file_data) {
            return Ok(Some(self.parse_items(&items)));
        }

        Ok(None)
    }

    /// Parse APE tag from file data
    fn parse_ape_tag(&self, data: &[u8]) -> Option<(ApeTagHeader, Vec<ApeTagItem>)> {
        // Minimum file size: footer (32 bytes)
        if data.len() < 32 {
            return None;
        }

        // Check for APE tag footer at end of file
        let footer_start = data.len() - 32;

        // Check signature
        if &data[footer_start..footer_start + 8] != APE_SIGNATURE {
            return None;
        }

        // Parse footer
        let header = self.parse_tag_header(&data[footer_start..])?;

        // Check if this is a footer (not header)
        if (header.flags & flags::IS_HEADER) != 0 {
            return None; // This is a header, not a footer
        }

        // Calculate tag start position
        let tag_size = header.tag_size as usize;
        let tag_start = footer_start + 32 - tag_size;

        // Parse items
        let mut items = Vec::new();
        let mut pos = tag_start;

        for _ in 0..header.item_count {
            if let Some(item) = self.parse_item(data, pos) {
                pos += 8 + item.key.len() + 1 + item.size as usize;
                items.push(item);
            } else {
                break;
            }
        }

        Some((header, items))
    }

    /// Parse APE tag header/footer
    fn parse_tag_header(&self, data: &[u8]) -> Option<ApeTagHeader> {
        if data.len() < 32 {
            return None;
        }

        let version = u32::from_le_bytes(data[8..12].try_into().unwrap());
        let tag_size = u32::from_le_bytes(data[12..16].try_into().unwrap());
        let item_count = u32::from_le_bytes(data[16..20].try_into().unwrap());
        let flags = u32::from_le_bytes(data[20..24].try_into().unwrap());
        let reserved = [
            data[24], data[25], data[26], data[27],
            data[28], data[29], data[30], data[31],
        ];

        Some(ApeTagHeader {
            version,
            tag_size,
            item_count,
            flags,
            reserved,
        })
    }

    /// Parse APE tag item
    fn parse_item(&self, data: &[u8], pos: usize) -> Option<ApeTagItem> {
        if pos + 8 > data.len() {
            return None;
        }

        let size = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
        let flags = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap());

        // Find null-terminated key
        let mut key_end = pos + 8;
        while key_end < data.len() && data[key_end] != 0 {
            key_end += 1;
        }

        if key_end >= data.len() {
            return None;
        }

        let key = String::from_utf8_lossy(&data[pos + 8..key_end]).to_string();
        let value_start = key_end + 1;
        let value_end = (value_start + size as usize).min(data.len());
        let value = data[value_start..value_end].to_vec();

        Some(ApeTagItem {
            size,
            flags,
            key,
            value,
        })
    }

    /// Parse items into metadata
    fn parse_items(&self, items: &[ApeTagItem]) -> ApeMetadata {
        let mut metadata = ApeMetadata::default();

        for item in items {
            let value = if item.value.is_empty() {
                String::new()
            } else {
                String::from_utf8_lossy(&item.value).trim_end_matches('\0').to_string()
            };

            match item.key.as_str() {
                fields::TITLE => metadata.title = Some(value),
                fields::ARTIST => metadata.artist = Some(value),
                fields::ALBUM => metadata.album = Some(value),
                fields::YEAR => metadata.year = Some(value),
                fields::TRACK => metadata.track = Some(value),
                fields::GENRE => metadata.genre = Some(value),
                fields::COMMENT => metadata.comment = Some(value),
                fields::LYRICS => metadata.lyrics = Some(value),
                _ => {}
            }
        }

        metadata
    }

    /// Write metadata to APE file (reserved for future use)
    #[allow(dead_code)]
    pub fn write_metadata(&self, _metadata: &ApeMetadata) -> std::io::Result<()> {
        // For APE, we would need to rebuild the tag at the end of the file
        // This is a simplified implementation

        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "APE metadata writing not yet implemented"
        ))
    }
}

/// APE metadata structure
#[derive(Debug, Clone, Default)]
pub struct ApeMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<String>,
    pub track: Option<String>,
    pub genre: Option<String>,
    pub comment: Option<String>,
    pub lyrics: Option<String>,
}

/// Detect if file is APE format
pub fn is_ape_file(path: &str) -> bool {
    if let Ok(file_data) = std::fs::read(path) {
        // APE files have MAC signature at beginning
        // Check for APE tag footer at end (more reliable)
        if file_data.len() >= 32 {
            let footer_start = file_data.len() - 32;
            if &file_data[footer_start..footer_start + 8] == APE_SIGNATURE {
                // Check version
                let version = u32::from_le_bytes(
                    file_data[footer_start + 8..footer_start + 12].try_into().unwrap()
                );
                return version == APE_VERSION;
            }
        }
    }
    false
}
