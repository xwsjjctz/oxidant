// FLAC metadata handling module

pub mod metadata;
pub mod vorbis;
pub mod picture;

pub use metadata::{FlacMetadataBlock, FlacMetadataBlockType, FLAC_SIGNATURE};
pub use vorbis::VorbisComment;
pub use vorbis::VorbisFields;
pub use picture::FlacPicture;