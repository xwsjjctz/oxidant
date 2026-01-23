// MP4/M4A/AAC format support
//
// MP4 format uses "atoms" (boxes) to store metadata.
// iTunes-style metadata is stored in ilst atom under moov/udta/meta.
//
// MP4 File Structure:
// - ftyp: File type atom
// - moov: Movie atom (container)
//   - udta: User data atom
//     - meta: Metadata atom (starts with 4 bytes of zeros + size + type)
//       - ilst: Information list atom (contains metadata items)
// - mdat: Media data atom
//
// Common iTunes metadata keys (4 bytes each):
// - ©nam: Title (title) - [0xA9, n, a, m]
// - ©ART: Artist (artist) - [0xA9, A, R, T]
// - ©alb: Album (album) - [0xA9, a, l, b]
// - ©day: Year (year) - [0xA9, d, a, y]
// - trkn: Track number (track)
// - ©gen: Genre (genre) - [0xA9, g, e, n]
// - ©cmt: Comment (comment) - [0xA9, c, m, t]
// - ©lyr: Lyrics (lyrics) - [0xA9, l, y, r]
// - covr: Cover art (cover)

use std::io::Read;
use std::fs::File;

pub const MP4_SIGNATURE: &[u8; 4] = b"ftyp";

// MP4 atom types
#[allow(dead_code)]
pub mod atoms {
    #[allow(dead_code)]
    pub const FTYP: &[u8; 4] = b"ftyp";
    #[allow(dead_code)]
    pub const MOOV: &[u8; 4] = b"moov";
    #[allow(dead_code)]
    pub const UDTA: &[u8; 4] = b"udta";
    pub const META: &[u8; 4] = b"meta";
    pub const ILST: &[u8; 4] = b"ilst";
    #[allow(dead_code)]
    pub const MDAT: &[u8; 4] = b"mdat";
    pub const DATA: &[u8; 4] = b"data";

    // iTunes metadata keys
    pub const TITLE: &[u8; 4] = &[0xA9, b'n', b'a', b'm']; // ©nam
    pub const ARTIST: &[u8; 4] = &[0xA9, b'A', b'R', b'T']; // ©ART
    pub const ALBUM: &[u8; 4] = &[0xA9, b'a', b'l', b'b']; // ©alb
    pub const YEAR: &[u8; 4] = &[0xA9, b'd', b'a', b'y']; // ©day
    pub const TRACK: &[u8; 4] = b"trkn";
    pub const GENRE: &[u8; 4] = &[0xA9, b'g', b'e', b'n']; // ©gen
    pub const COMMENT: &[u8; 4] = &[0xA9, b'c', b'm', b't']; // ©cmt
    pub const LYRICS: &[u8; 4] = &[0xA9, b'l', b'y', b'r']; // ©lyr
    pub const COVER: &[u8; 4] = b"covr";
}

/// MP4 atom header (reserved for future use)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Mp4AtomHeader {
    pub offset: usize,
    pub size: u64,
    pub atom_type: [u8; 4],
    pub is_extended: bool,
}

/// MP4 metadata handler
pub struct Mp4File {
    pub path: String,
}

impl Mp4File {
    /// Create a new MP4 file handler
    pub fn new(path: String) -> Self {
        Mp4File { path }
    }

    /// Read metadata from MP4 file
    pub fn read_metadata(&self) -> std::io::Result<Option<Mp4Metadata>> {
        let file_data = std::fs::read(&self.path)?;

        // Find ilst atom
        if let Some(ilst_data) = self.find_ilst_atom(&file_data) {
            Ok(Some(self.parse_ilst(&ilst_data)))
        } else {
            Ok(None)
        }
    }

    /// Find ilst atom in MP4 file data
    fn find_ilst_atom(&self, data: &[u8]) -> Option<Vec<u8>> {
        let mut pos = 0;

        while pos < data.len() {
            if pos + 8 > data.len() {
                break;
            }

            let size = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as u64;
            let atom_type = [data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]];

            // Handle extended size (64-bit)
            let actual_size = if size == 1 {
                if pos + 16 > data.len() {
                    break;
                }
                u64::from_be_bytes(data[pos + 8..pos + 16].try_into().unwrap())
            } else {
                size as u64
            };

            let atom_end = pos + actual_size as usize;

            // Check for meta atom (skip the 4-byte zero prefix)
            if atom_type == *atoms::META {
                // meta atom starts with 4 bytes of zeros
                let meta_pos = if pos + 8 + 4 <= data.len() {
                    pos + 8 + 4
                } else {
                    pos + 8
                };

                // Search for ilst within meta
                let mut inner_pos = meta_pos;
                while inner_pos < data.len().min(atom_end) {
                    if inner_pos + 8 > data.len() {
                        break;
                    }

                    let inner_size = u32::from_be_bytes(data[inner_pos..inner_pos + 4].try_into().unwrap()) as u64;
                    let inner_type = [data[inner_pos + 4], data[inner_pos + 5], data[inner_pos + 6], data[inner_pos + 7]];

                    if inner_type == *atoms::ILST {
                        // Return ilst content (skip header)
                        let ilist_start = inner_pos + 8;
                        let ilist_end = (inner_pos + inner_size as usize).min(data.len());
                        return Some(data[ilist_start..ilist_end].to_vec());
                    }

                    let inner_actual_size = if inner_size == 1 {
                        inner_pos + 16 + (u64::from_be_bytes(data[inner_pos + 12..inner_pos + 20].try_into().unwrap()) as usize)
                    } else {
                        inner_pos + inner_size as usize
                    };

                    inner_pos = inner_actual_size;
                }
            }

            pos = atom_end;
        }

        None
    }

    /// Parse ilst atom data
    fn parse_ilst(&self, data: &[u8]) -> Mp4Metadata {
        let mut metadata = Mp4Metadata::default();
        let mut pos = 0;

        while pos < data.len() {
            if pos + 8 > data.len() {
                break;
            }

            let atom_size = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
            let atom_type = [data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]];

            // Extract data atom content
            let data_pos = pos + 8; // Skip item atom header
            if data_pos + 8 > data.len() {
                break;
            }

            // Check for data atom
            let data_atom_type = [data[data_pos + 4], data[data_pos + 5], data[data_pos + 6], data[data_pos + 7]];
            if data_atom_type == *atoms::DATA {
                // Data atom structure: size(4) + type(4) + reserved(4) + data
                let content_start = data_pos + 16;
                let content_end = (pos + atom_size).min(data.len());

                if content_start < content_end {
                    let content = &data[content_start..content_end];

                    // Map atom type to metadata field
                    if atom_type == *atoms::TITLE {
                        metadata.title = Some(String::from_utf8_lossy(content).trim_end_matches('\0').to_string());
                    } else if atom_type == *atoms::ARTIST {
                        metadata.artist = Some(String::from_utf8_lossy(content).trim_end_matches('\0').to_string());
                    } else if atom_type == *atoms::ALBUM {
                        metadata.album = Some(String::from_utf8_lossy(content).trim_end_matches('\0').to_string());
                    } else if atom_type == *atoms::YEAR {
                        metadata.year = Some(String::from_utf8_lossy(content).trim_end_matches('\0').to_string());
                    } else if atom_type == *atoms::TRACK {
                        // Track number is stored as 2 bytes: track number / total tracks
                        if content.len() >= 6 {
                            let track_num = u16::from_be_bytes([content[2], content[3]]);
                            metadata.track = Some(track_num.to_string());
                        }
                    } else if atom_type == *atoms::GENRE {
                        metadata.genre = Some(String::from_utf8_lossy(content).trim_end_matches('\0').to_string());
                    } else if atom_type == *atoms::COMMENT {
                        metadata.comment = Some(String::from_utf8_lossy(content).trim_end_matches('\0').to_string());
                    } else if atom_type == *atoms::LYRICS {
                        metadata.lyrics = Some(String::from_utf8_lossy(content).trim_end_matches('\0').to_string());
                    } else if atom_type == *atoms::COVER {
                        metadata.cover = Some(content.to_vec());
                    }
                }
            }

            pos += atom_size;
        }

        metadata
    }

    /// Write metadata to MP4 file (reserved for future use)
    #[allow(dead_code)]
    pub fn write_metadata(&self, metadata: &Mp4Metadata) -> std::io::Result<()> {
        // For MP4, we would need to rebuild the ilst atom
        // This is a simplified implementation that preserves existing structure
        // A full implementation would need to handle complex atom tree manipulation

        // Read the entire file
        let file_data = std::fs::read(&self.path)?;

        // For now, this is a placeholder - full implementation would
        // parse the atom tree, modify ilst, and rebuild the file
        let _ = (file_data, metadata);

        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "MP4 metadata writing not yet implemented"
        ))
    }
}

/// MP4 metadata structure
#[derive(Debug, Clone, Default)]
pub struct Mp4Metadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<String>,
    pub track: Option<String>,
    pub genre: Option<String>,
    pub comment: Option<String>,
    pub lyrics: Option<String>,
    pub cover: Option<Vec<u8>>,
}

/// Detect if file is MP4/M4A format
#[allow(dead_code)]
pub fn is_mp4_file(path: &str) -> bool {
    if let Ok(mut file) = File::open(path) {
        let mut signature = [0u8; 4];
        if file.read_exact(&mut signature).is_ok() {
            // Check for ftyp atom (MP4 files start with ftyp)
            return signature == *MP4_SIGNATURE;
        }
    }
    false
}

/// Read MP4 atom header at position (reserved for future use)
#[allow(dead_code)]
pub fn read_atom_header(data: &[u8], pos: usize) -> Option<Mp4AtomHeader> {
    if pos + 8 > data.len() {
        return None;
    }

    let size = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as u64;
    let atom_type = [data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]];
    let is_extended = size == 1;

    let actual_size = if is_extended {
        if pos + 16 > data.len() {
            return None;
        }
        u64::from_be_bytes(data[pos + 8..pos + 16].try_into().unwrap())
    } else {
        size
    };

    Some(Mp4AtomHeader {
        offset: pos,
        size: actual_size,
        atom_type,
        is_extended,
    })
}
