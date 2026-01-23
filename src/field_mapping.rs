// Unified metadata field mapping system
//
// This module provides a unified interface for mapping metadata fields
// between different audio formats (ID3, FLAC, OGG, MP4, APE, etc.)
//
// Each format has its own field names and conventions:
// - ID3v2: Frame IDs (TIT2, TPE1, TALB, etc.)
// - FLAC/OGG: Vorbis Comment keys (TITLE, ARTIST, ALBUM, etc.)
// - MP4: iTunes atoms (©nam, ©ART, ©alb, etc.)
// - APE: Tag field names (Title, Artist, Album, etc.)
//
// This module standardizes field access across formats.

use std::collections::HashMap;

/// Standard metadata fields
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StandardField {
    Title,
    Artist,
    Album,
    Year,
    Track,
    Genre,
    Comment,
    Lyrics,
    Cover,
}

impl StandardField {
    /// Get standard field name (lowercase)
    pub fn as_str(&self) -> &'static str {
        match self {
            StandardField::Title => "title",
            StandardField::Artist => "artist",
            StandardField::Album => "album",
            StandardField::Year => "year",
            StandardField::Track => "track",
            StandardField::Genre => "genre",
            StandardField::Comment => "comment",
            StandardField::Lyrics => "lyrics",
            StandardField::Cover => "cover",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "title" => Some(StandardField::Title),
            "artist" => Some(StandardField::Artist),
            "album" => Some(StandardField::Album),
            "year" => Some(StandardField::Year),
            "track" => Some(StandardField::Track),
            "genre" => Some(StandardField::Genre),
            "comment" => Some(StandardField::Comment),
            "lyrics" => Some(StandardField::Lyrics),
            "cover" => Some(StandardField::Cover),
            _ => None,
        }
    }
}

/// Format-specific field mappings
pub struct FieldMappings;

impl FieldMappings {
    // ID3v2 frame IDs
    pub const ID3V2_TITLE: &str = "TIT2";
    pub const ID3V2_ARTIST: &str = "TPE1";
    pub const ID3V2_ALBUM: &str = "TALB";
    pub const ID3V2_YEAR: &str = "TDRC";
    pub const ID3V2_TRACK: &str = "TRCK";
    pub const ID3V2_GENRE: &str = "TCON";
    pub const ID3V2_COMMENT: &str = "COMM";
    pub const ID3V2_LYRICS: &str = "USLT";
    pub const ID3V2_COVER: &str = "APIC";

    // Vorbis Comment keys (FLAC/OGG)
    pub const VORBIS_TITLE: &str = "TITLE";
    pub const VORBIS_ARTIST: &str = "ARTIST";
    pub const VORBIS_ALBUM: &str = "ALBUM";
    pub const VORBIS_YEAR: &str = "DATE";
    pub const VORBIS_TRACK: &str = "TRACKNUMBER";
    pub const VORBIS_GENRE: &str = "GENRE";
    pub const VORBIS_COMMENT: &str = "COMMENT";
    pub const VORBIS_LYRICS: &str = "LYRICS";

    // MP4 iTunes atoms (with special characters)
    pub const MP4_TITLE: &[u8; 4] = b"\xA9nam"; // ©nam
    pub const MP4_ARTIST: &[u8; 4] = b"\xA9ART"; // ©ART
    pub const MP4_ALBUM: &[u8; 4] = b"\xA9alb"; // ©alb
    pub const MP4_YEAR: &[u8; 4] = b"\xA9day"; // ©day
    pub const MP4_TRACK: &[u8; 4] = b"trkn";
    pub const MP4_GENRE: &[u8; 4] = b"\xA9gen"; // ©gen
    pub const MP4_COMMENT: &[u8; 4] = b"\xA9cmt"; // ©cmt
    pub const MP4_LYRICS: &[u8; 4] = b"\xA9lyr"; // ©lyr
    pub const MP4_COVER: &[u8; 4] = b"covr";

    // APE tag fields
    pub const APE_TITLE: &str = "Title";
    pub const APE_ARTIST: &str = "Artist";
    pub const APE_ALBUM: &str = "Album";
    pub const APE_YEAR: &str = "Year";
    pub const APE_TRACK: &str = "Track";
    pub const APE_GENRE: &str = "Genre";
    pub const APE_COMMENT: &str = "Comment";
    pub const APE_LYRICS: &str = "Lyrics";

    /// Get ID3v2 frame ID for a standard field
    pub fn to_id3v2(field: &StandardField) -> &'static str {
        match field {
            StandardField::Title => Self::ID3V2_TITLE,
            StandardField::Artist => Self::ID3V2_ARTIST,
            StandardField::Album => Self::ID3V2_ALBUM,
            StandardField::Year => Self::ID3V2_YEAR,
            StandardField::Track => Self::ID3V2_TRACK,
            StandardField::Genre => Self::ID3V2_GENRE,
            StandardField::Comment => Self::ID3V2_COMMENT,
            StandardField::Lyrics => Self::ID3V2_LYRICS,
            StandardField::Cover => Self::ID3V2_COVER,
        }
    }

    /// Get Vorbis Comment key for a standard field
    pub fn to_vorbis(field: &StandardField) -> &'static str {
        match field {
            StandardField::Title => Self::VORBIS_TITLE,
            StandardField::Artist => Self::VORBIS_ARTIST,
            StandardField::Album => Self::VORBIS_ALBUM,
            StandardField::Year => Self::VORBIS_YEAR,
            StandardField::Track => Self::VORBIS_TRACK,
            StandardField::Genre => Self::VORBIS_GENRE,
            StandardField::Comment => Self::VORBIS_COMMENT,
            StandardField::Lyrics => Self::VORBIS_LYRICS,
            StandardField::Cover => "COVERART", // Non-standard but commonly used
        }
    }

    /// Get APE tag field for a standard field
    pub fn to_ape(field: &StandardField) -> &'static str {
        match field {
            StandardField::Title => Self::APE_TITLE,
            StandardField::Artist => Self::APE_ARTIST,
            StandardField::Album => Self::APE_ALBUM,
            StandardField::Year => Self::APE_YEAR,
            StandardField::Track => Self::APE_TRACK,
            StandardField::Genre => Self::APE_GENRE,
            StandardField::Comment => Self::APE_COMMENT,
            StandardField::Lyrics => Self::APE_LYRICS,
            StandardField::Cover => "Cover Art (Front)",
        }
    }

    /// Convert ID3v2 frame to standard field
    pub fn from_id3v2(frame_id: &str) -> Option<StandardField> {
        match frame_id {
            Self::ID3V2_TITLE => Some(StandardField::Title),
            Self::ID3V2_ARTIST => Some(StandardField::Artist),
            Self::ID3V2_ALBUM => Some(StandardField::Album),
            Self::ID3V2_YEAR | "TYER" => Some(StandardField::Year), // Also support legacy TYER
            Self::ID3V2_TRACK => Some(StandardField::Track),
            Self::ID3V2_GENRE => Some(StandardField::Genre),
            Self::ID3V2_COMMENT => Some(StandardField::Comment),
            Self::ID3V2_LYRICS => Some(StandardField::Lyrics),
            Self::ID3V2_COVER => Some(StandardField::Cover),
            _ => None,
        }
    }

    /// Convert Vorbis Comment key to standard field
    pub fn from_vorbis(key: &str) -> Option<StandardField> {
        match key.to_uppercase().as_str() {
            Self::VORBIS_TITLE => Some(StandardField::Title),
            Self::VORBIS_ARTIST => Some(StandardField::Artist),
            Self::VORBIS_ALBUM => Some(StandardField::Album),
            Self::VORBIS_YEAR | "YEAR" => Some(StandardField::Year), // Also support YEAR
            Self::VORBIS_TRACK | "TRACKNUMBER" => Some(StandardField::Track),
            Self::VORBIS_GENRE => Some(StandardField::Genre),
            Self::VORBIS_COMMENT => Some(StandardField::Comment),
            Self::VORBIS_LYRICS => Some(StandardField::Lyrics),
            "COVERART" | "COVER" => Some(StandardField::Cover),
            _ => None,
        }
    }

    /// Convert APE tag field to standard field
    pub fn from_ape(key: &str) -> Option<StandardField> {
        match key {
            Self::APE_TITLE => Some(StandardField::Title),
            Self::APE_ARTIST => Some(StandardField::Artist),
            Self::APE_ALBUM => Some(StandardField::Album),
            Self::APE_YEAR => Some(StandardField::Year),
            Self::APE_TRACK => Some(StandardField::Track),
            Self::APE_GENRE => Some(StandardField::Genre),
            Self::APE_COMMENT => Some(StandardField::Comment),
            Self::APE_LYRICS => Some(StandardField::Lyrics),
            "Cover Art (Front)" | "COVER ART (FRONT)" => Some(StandardField::Cover),
            _ => None,
        }
    }
}

/// Metadata value converter for handling format-specific value formats
pub struct ValueConverter;

impl ValueConverter {
    /// Convert year string to various formats
    pub fn normalize_year(year: &str) -> String {
        // Extract 4-digit year from various formats
        let year_str = year.trim();
        if year_str.len() >= 4 {
            year_str[..4].to_string()
        } else {
            year_str.to_string()
        }
    }

    /// Convert track number to standard format (e.g., "1/10" -> "1")
    pub fn normalize_track(track: &str) -> String {
        track.split('/').next().unwrap_or(track).to_string()
    }

    /// Parse genre from numeric ID3v1 genre (if applicable)
    pub fn parse_genre_id3v1(genre_id: u8) -> Option<&'static str> {
        // TODO: Implement ID3v1 genre lookup table
        // This would map genre IDs (0-255) to genre names
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_field_parsing() {
        assert_eq!(StandardField::from_str("title"), Some(StandardField::Title));
        assert_eq!(StandardField::from_str("TITLE"), Some(StandardField::Title));
        assert_eq!(StandardField::from_str("TiTlE"), Some(StandardField::Title));
        assert_eq!(StandardField::from_str("unknown"), None);
    }

    #[test]
    fn test_field_mapping() {
        assert_eq!(FieldMappings::to_id3v2(&StandardField::Title), "TIT2");
        assert_eq!(FieldMappings::to_vorbis(&StandardField::Title), "TITLE");
        assert_eq!(FieldMappings::to_ape(&StandardField::Title), "Title");

        assert_eq!(FieldMappings::from_id3v2("TIT2"), Some(StandardField::Title));
        assert_eq!(FieldMappings::from_vorbis("TITLE"), Some(StandardField::Title));
        assert_eq!(FieldMappings::from_ape("Title"), Some(StandardField::Title));
    }

    #[test]
    fn test_value_normalization() {
        assert_eq!(ValueConverter::normalize_year("2024-01-15"), "2024");
        assert_eq!(ValueConverter::normalize_year("2024"), "2024");
        assert_eq!(ValueConverter::normalize_track("1/10"), "1");
        assert_eq!(ValueConverter::normalize_track("5"), "5");
    }
}
