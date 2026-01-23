// FLAC metadata handling module

pub mod metadata;
pub mod vorbis;
pub mod picture;

pub use metadata::{FlacMetadataBlock, FlacMetadataBlockType, FLAC_SIGNATURE};
// Note: VorbisComment, VorbisFields, and FlacPicture are exported but may be unused in current version
// They are kept for API compatibility and future use
#[allow(unused_imports)]
pub use vorbis::VorbisComment;
#[allow(unused_imports)]
pub use vorbis::VorbisFields;
#[allow(unused_imports)]
pub use picture::FlacPicture;