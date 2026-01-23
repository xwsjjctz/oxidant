// MP4/M4A/AAC format support
//
// MP4 format uses "atoms" (boxes) to store metadata.
// iTunes-style metadata is stored in ilst atom under moov/udta/meta.
//
// TODO: Implement MP4/M4A format support
//
// MP4 File Structure:
// - ftyp: File type atom
// - moov: Movie atom (container)
//   - udta: User data atom
//     - meta: Metadata atom
//       - ilst: Information list atom (contains metadata items)
// - mdat: Media data atom
//
// Common iTunes metadata keys:
// - ©nam: Title (title)
// - ©ART: Artist (artist)
// - ©alb: Album (album)
// - ©day: Year (year)
// - trkn: Track number (track)
// - ©gen: Genre (genre)
// - ©cmt: Comment (comment)
// - ©lyr: Lyrics (lyrics)
// - covr: Cover art (cover)
//
// Tasks to complete:
// 1. Create MP4 atom parsing infrastructure
// 2. Implement atom reading (size + type + data)
// 3. Find and parse ilst atom
// 4. Implement metadata field mapping (iTunes keys -> standard fields)
// 5. Handle cover art in covr atom
// 6. Implement metadata writing (atom replacement)
// 7. Add MP4 signature detection (ftyp atom)
// 8. Integrate with lib.rs
//
// Reference:
// - ISO/IEC 14496-12: ISO Base Media File Format
// - https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/Metadata/Metadata.html

pub const MP4_SIGNATURE: &[u8; 4] = b"ftyp";

/// MP4 metadata handler (TODO: implement)
pub struct Mp4File {
    pub path: String,
}

impl Mp4File {
    pub fn new(path: String) -> Self {
        Mp4File { path }
    }

    // TODO: Implement metadata reading
    // pub fn read_metadata(&self) -> std::io::Result<Metadata> { ... }

    // TODO: Implement metadata writing
    // pub fn write_metadata(&self, metadata: &Metadata) -> std::io::Result<()> { ... }

    // TODO: Implement cover art reading
    // pub fn read_cover(&self) -> std::io::Result<Option<CoverArt>> { ... }

    // TODO: Implement cover art writing
    // pub fn write_cover(&self, cover: &CoverArt) -> std::io::Result<()> { ... }
}

/// Detect if file is MP4/M4A format
pub fn is_mp4_file(path: &str) -> bool {
    // TODO: Implement MP4 detection
    // MP4 files start with ftyp atom
    false
}

/// MP4 atom types (for future implementation)
pub mod atoms {
    pub const FTYP: &[u8; 4] = b"ftyp";
    pub const MOOV: &[u8; 4] = b"moov";
    pub const UDTA: &[u8; 4] = b"udta";
    pub const META: &[u8; 4] = b"meta";
    pub const ILST: &[u8; 4] = b"ilst";
    pub const MDAT: &[u8; 4] = b"mdat";

    // iTunes metadata keys
    pub const TITLE: &[u8; 4] = b"\xA9nam"; // ©nam
    pub const ARTIST: &[u8; 4] = b"\xA9ART"; // ©ART
    pub const ALBUM: &[u8; 4] = b"\xA9alb"; // ©alb
    pub const YEAR: &[u8; 4] = b"\xA9day"; // ©day
    pub const TRACK: &[u8; 4] = b"trkn";
    pub const GENRE: &[u8; 4] = b"\xA9gen"; // ©gen
    pub const COMMENT: &[u8; 4] = b"\xA9cmt"; // ©cmt
    pub const LYRICS: &[u8; 4] = b"\xA9lyr"; // ©lyr
    pub const COVER: &[u8; 4] = b"covr";
}
