// ID3 metadata handling module
pub mod v1;
pub mod v2;
pub mod frames;

pub use v1::Id3v1Tag;
pub use v2::Id3v2Tag;