// OGG Vorbis metadata support
//
// OGG File Structure:
// - OGG Page Header (27 bytes)
//   - Capture Pattern: "OggS" (4 bytes)
//   - Version: 0 (1 byte)
//   - Header Type: 1=continuation, 2=bos, 4=eos (1 byte)
//   - Granule Position (8 bytes)
//   - Bitstream Serial Number (4 bytes)
//   - Page Sequence Number (4 bytes)
//   - CRC Checksum (4 bytes)
//   - Number of Page Segments (1 byte)
//   - Segment Table (variable)
//
// Vorbis Structure:
// 1. Identification Header (first page)
// 2. Comment Header (second page) - Contains Vorbis Comment
// 3. Setup Header (third page)
// 4. Audio Data pages

pub mod vorbis;
pub mod page;

// Re-export VorbisComment for external use (reserved for future use)
#[allow(unused_imports)]
pub use vorbis::VorbisComment;

// OGG signature
pub const OGG_SIGNATURE: &[u8; 4] = b"OggS";

// OGG page header types (used internally)
#[allow(dead_code)]
pub(crate) const OGG_HEADER_TYPE_CONTINUATION: u8 = 0x01;
#[allow(dead_code)]
pub(crate) const OGG_HEADER_TYPE_BOS: u8 = 0x02; // Beginning of Stream
#[allow(dead_code)]
pub(crate) const OGG_HEADER_TYPE_EOS: u8 = 0x04; // End of Stream
