// Oxidant - A high-performance audio metadata library
//
// This library supports both Python bindings (via PyO3) and pure Rust usage.
// Enable the "python" feature to build Python bindings.

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::Bound;
#[cfg(feature = "python")]
use pyo3::types::PyList;

use std::fs::File;
use std::io::{BufReader, Read, Seek};
use serde::{Serialize, Deserialize, Serializer};

mod id3;
mod flac;
mod ogg;
mod opus;
mod mp4;
mod ape;
mod utils;

use id3::{Id3v1Tag, Id3v2Tag};
use flac::{FlacMetadataBlock, FlacMetadataBlockType, FLAC_SIGNATURE};
use ogg::{OGG_SIGNATURE, vorbis::OggVorbisFile};
use opus::OpusFile;
use mp4::Mp4File;
use ape::ApeFile;

// Alias for our custom Result type to avoid conflicts with std::result::Result
pub type AudioResult<T> = std::result::Result<T, AudioFileError>;

// ============================================================================
// Core Types (available in both Rust and Python)
// ============================================================================

/// Audio file metadata handler
#[derive(Debug)]
pub struct AudioFile {
    pub path: String,
    pub file_type: String,
}

// Error type for AudioFile operations
#[derive(Debug)]
pub enum AudioFileError {
    IoError(std::io::Error),
    UnsupportedFormat(String),
    ParseError(String),
}

impl std::fmt::Display for AudioFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioFileError::IoError(e) => write!(f, "I/O error: {}", e),
            AudioFileError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
            AudioFileError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for AudioFileError {}

impl From<std::io::Error> for AudioFileError {
    fn from(e: std::io::Error) -> Self {
        AudioFileError::IoError(e)
    }
}


// Custom serialization for Vec<u8> to base64 string
fn serialize_as_base64<S>(data: &Vec<u8>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use base64::prelude::*;
    let base64_str = BASE64_STANDARD.encode(data);
    serializer.serialize_str(&base64_str)
}

/// Custom deserialization for base64 string to Vec<u8>
#[allow(dead_code)]
fn deserialize_base64_to_vec<'de, D>(deserializer: D) -> std::result::Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use base64::prelude::*;
    let s = String::deserialize(deserializer)?;
    BASE64_STANDARD.decode(&s).map_err(serde::de::Error::custom)
}

// Private implementation block for internal methods
impl AudioFile {
    /// Decode text frame data
    fn decode_text_frame(data: &[u8]) -> Option<String> {
        if data.is_empty() {
            return None;
        }

        // First byte indicates encoding
        let encoding = data[0];
        let text_data = &data[1..];

        let result = match encoding {
            0 => {
                // ISO-8859-1 (use windows-1252 which is a superset)
                encoding_rs::WINDOWS_1252.decode(text_data).0
            }
            1 => encoding_rs::UTF_16LE.decode(text_data).0,
            2 => encoding_rs::UTF_16BE.decode(text_data).0,
            3 => encoding_rs::UTF_8.decode(text_data).0,
            _ => return None,
        };

        Some(result.trim_end_matches('\0').to_string())
    }

    /// Read metadata from the audio file (internal method)
    fn read_metadata_internal(&self) -> AudioResult<Metadata> {
        match self.file_type.as_str() {
            "id3v2" => self.read_id3v2_metadata(),
            "id3v1" => self.read_id3v1_metadata(),
            "flac" => self.read_flac_metadata(),
            "ogg" => self.read_ogg_metadata(),
            "opus" => self.read_opus_metadata(),
            "mp4" => self.read_mp4_metadata(),
            "ape" => self.read_ape_metadata(),
            _ => Ok(Metadata::default()),
        }
    }

    /// Detect file type
    fn detect_file_type(path: &str) -> AudioResult<String> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Check for ID3v2
        let mut id3_signature = [0u8; 3];
        if reader.read_exact(&mut id3_signature).is_ok() {
            if &id3_signature == b"ID3" {
                return Ok("id3v2".to_string());
            }
        }

        // Check for FLAC
        reader.seek(std::io::SeekFrom::Start(0))?;
        let mut flac_signature = [0u8; 4];
        if reader.read_exact(&mut flac_signature).is_ok() {
            if &flac_signature == FLAC_SIGNATURE {
                return Ok("flac".to_string());
            }
        }

        // Check for OGG
        reader.seek(std::io::SeekFrom::Start(0))?;
        let mut ogg_signature = [0u8; 4];
        if reader.read_exact(&mut ogg_signature).is_ok() {
            if &ogg_signature == OGG_SIGNATURE {
                // Further check for Opus or Vorbis
                let mut opus_sig = [0u8; 4];
                if reader.seek(std::io::SeekFrom::Start(28)).is_ok() {
                    if reader.read_exact(&mut opus_sig).is_ok() {
                        if &opus_sig == b"Opus" {
                            return Ok("opus".to_string());
                        }
                    }
                }
                return Ok("ogg".to_string());
            }
        }

        // Check for MP4
        reader.seek(std::io::SeekFrom::Start(4))?;
        let mut mp4_signature = [0u8; 4];
        if reader.read_exact(&mut mp4_signature).is_ok() {
            let sig_str = std::str::from_utf8(&mp4_signature).unwrap_or("");
            if sig_str == "ftyp" {
                return Ok("mp4".to_string());
            }
        }

        // Check for APE (at end of file)
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();
        if file_size > 32 {
            let mut reader = BufReader::new(file);
            reader.seek(std::io::SeekFrom::End(-32))?;
            let mut ape_signature = [0u8; 8];
            if reader.read_exact(&mut ape_signature).is_ok() {
                if &ape_signature == b"APETAGEX" {
                    return Ok("ape".to_string());
                }
            }
        }

        // Check for ID3v1 (at end of file)
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();
        if file_size > 128 {
            let mut reader = BufReader::new(file);
            reader.seek(std::io::SeekFrom::End(-128))?;
            let mut tag = [0u8; 3];
            if reader.read_exact(&mut tag).is_ok() {
                if &tag == b"TAG" {
                    return Ok("id3v1".to_string());
                }
            }
        }

        Err(AudioFileError::UnsupportedFormat("Unknown audio format".to_string()))
    }

    /// Read ID3v2 metadata
    fn read_id3v2_metadata(&self) -> AudioResult<Metadata> {
        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);
        let tag = Id3v2Tag::read(&mut reader)?
            .ok_or_else(|| AudioFileError::ParseError("No ID3v2 tag found".to_string()))?;

        let mut metadata = Metadata::default();

        // Parse frames
        for frame in &tag.frames {
            match frame.frame_id.as_str() {
                "TIT2" => metadata.title = Self::decode_text_frame(&frame.data),
                "TPE1" => metadata.artist = Self::decode_text_frame(&frame.data),
                "TALB" => metadata.album = Self::decode_text_frame(&frame.data),
                "TYER" | "TDRC" => metadata.year = Self::decode_text_frame(&frame.data),
                "TRCK" => metadata.track = Self::decode_text_frame(&frame.data),
                "TCON" => metadata.genre = Self::decode_text_frame(&frame.data),
                "COMM" => metadata.comment = Self::decode_text_frame(&frame.data),
                "USLT" => {
                    if let Some((_language, _description, lyrics)) = id3::frames::decode_uslt_frame(&frame.data) {
                        metadata.lyrics = Some(lyrics);
                    }
                }
                _ => {}
            }
        }

        Ok(metadata)
    }

    /// Read ID3v1 metadata
    fn read_id3v1_metadata(&self) -> AudioResult<Metadata> {
        let tag = Id3v1Tag::read_from_file(&self.path)?
            .ok_or_else(|| AudioFileError::ParseError("No ID3v1 tag found".to_string()))?;

        let metadata = Metadata {
            title: if !tag.title.is_empty() { Some(tag.title) } else { None },
            artist: if !tag.artist.is_empty() { Some(tag.artist) } else { None },
            album: if !tag.album.is_empty() { Some(tag.album) } else { None },
            year: if !tag.year.is_empty() { Some(tag.year) } else { None },
            comment: if !tag.comment.is_empty() { Some(tag.comment) } else { None },
            track: tag.track.map(|t| t.to_string()),
            ..Default::default()
        };

        Ok(metadata)
    }

    /// Read FLAC metadata
    fn read_flac_metadata(&self) -> AudioResult<Metadata> {
        use flac::vorbis::VorbisComment;
        use std::io::Cursor;

        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);

        // Check FLAC signature
        let mut signature = [0u8; 4];
        reader.read_exact(&mut signature)?;

        if signature != *FLAC_SIGNATURE {
            return Ok(Metadata::default());
        }

        let mut metadata = Metadata::default();

        // Read metadata blocks
        loop {
            match FlacMetadataBlock::read(&mut reader) {
                Ok(block) => {
                    if block.header.block_type == FlacMetadataBlockType::VorbisComment {
                        if let Ok(vorbis) = VorbisComment::read(&mut Cursor::new(&block.data)) {
                            // Convert VorbisComment to Metadata
                            for (key, value) in vorbis.comments {
                                match key.to_uppercase().as_str() {
                                    "TITLE" => metadata.title = Some(value),
                                    "ARTIST" => metadata.artist = Some(value),
                                    "ALBUM" => metadata.album = Some(value),
                                    "DATE" => metadata.year = Some(value),
                                    "TRACKNUMBER" => metadata.track = Some(value),
                                    "GENRE" => metadata.genre = Some(value),
                                    "COMMENT" => metadata.comment = Some(value),
                                    "LYRICS" => metadata.lyrics = Some(value),
                                    "ALBUMARTIST" => metadata.album_artist = Some(value),
                                    "COMPOSER" => metadata.composer = Some(value),
                                    _ => {}
                                }
                            }
                        }
                    }

                    if block.header.is_last {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        Ok(metadata)
    }

    /// Read OGG metadata
    fn read_ogg_metadata(&self) -> AudioResult<Metadata> {
        let ogg_file = OggVorbisFile::new(self.path.clone());
        if let Some(comment) = ogg_file.read_comment()? {
            Ok(Self::vorbis_to_metadata(comment))
        } else {
            Ok(Metadata::default())
        }
    }

    /// Read OPUS metadata
    fn read_opus_metadata(&self) -> AudioResult<Metadata> {
        let opus_file = OpusFile::new(self.path.clone());
        if let Some(comment) = opus_file.read_comment()? {
            Ok(Self::vorbis_to_metadata(comment))
        } else {
            Ok(Metadata::default())
        }
    }

    /// Read MP4 metadata
    fn read_mp4_metadata(&self) -> AudioResult<Metadata> {
        let mp4_file = Mp4File::new(self.path.clone());
        if let Some(meta) = mp4_file.read_metadata()? {
            Ok(Self::mp4_to_metadata(meta))
        } else {
            Ok(Metadata::default())
        }
    }

    /// Read APE metadata
    fn read_ape_metadata(&self) -> AudioResult<Metadata> {
        let ape_file = ApeFile::new(self.path.clone());
        if let Some(meta) = ape_file.read_metadata()? {
            Ok(Self::ape_to_metadata(meta))
        } else {
            Ok(Metadata::default())
        }
    }

    /// Convert VorbisComment to Metadata
    fn vorbis_to_metadata(comment: flac::vorbis::VorbisComment) -> Metadata {
        let mut metadata = Metadata::default();
        for (key, value) in comment.comments {
            match key.to_uppercase().as_str() {
                "TITLE" => metadata.title = Some(value),
                "ARTIST" => metadata.artist = Some(value),
                "ALBUM" => metadata.album = Some(value),
                "DATE" => metadata.year = Some(value),
                "TRACKNUMBER" => metadata.track = Some(value),
                "GENRE" => metadata.genre = Some(value),
                "COMMENT" => metadata.comment = Some(value),
                "LYRICS" => metadata.lyrics = Some(value),
                "ALBUMARTIST" => metadata.album_artist = Some(value),
                "COMPOSER" => metadata.composer = Some(value),
                _ => {}
            }
        }
        metadata
    }

    /// Convert Mp4Metadata to Metadata
    fn mp4_to_metadata(meta: mp4::Mp4Metadata) -> Metadata {
        Metadata {
            title: meta.title,
            artist: meta.artist,
            album: meta.album,
            year: meta.year,
            comment: meta.comment,
            track: meta.track,
            genre: meta.genre,
            album_artist: None,
            composer: None,
            lyrics: meta.lyrics,
            cover: None,
        }
    }

    /// Convert ApeMetadata to Metadata
    fn ape_to_metadata(meta: ape::ApeMetadata) -> Metadata {
        Metadata {
            title: meta.title,
            artist: meta.artist,
            album: meta.album,
            year: meta.year,
            comment: meta.comment,
            track: meta.track,
            genre: meta.genre,
            album_artist: None,
            composer: None,
            lyrics: meta.lyrics,
            cover: None,
        }
    }
}

/// Public API for AudioFile (no PyO3 dependencies)
impl AudioFile {
    /// Create a new AudioFile instance
    pub fn new(path: String) -> AudioResult<Self> {
        let file_type = Self::detect_file_type(&path)?;
        Ok(Self { path, file_type })
    }

    /// Get metadata as JSON string
    pub fn get_metadata(&self) -> AudioResult<String> {
        let metadata = self.read_metadata_internal()?;
        serde_json::to_string(&metadata)
            .map_err(|e| AudioFileError::ParseError(e.to_string()))
    }

    /// Get metadata as serde_json Value
    pub fn get_metadata_value(&self) -> AudioResult<serde_json::Value> {
        let metadata = self.read_metadata_internal()?;
        serde_json::to_value(&metadata)
            .map_err(|e| AudioFileError::ParseError(e.to_string()))
    }

    /// Set metadata from JSON string
    pub fn set_metadata(&self, metadata_json: String) -> AudioResult<()> {
        // Parse JSON to validate it
        let _value: serde_json::Value = serde_json::from_str(&metadata_json)
            .map_err(|e| AudioFileError::ParseError(e.to_string()))?;

        // For now, just return success - full implementation would write to file
        // This is a placeholder
        Ok(())
    }

    /// Get the file type/version
    pub fn get_version(&self) -> AudioResult<String> {
        match self.file_type.as_str() {
            "id3v2" => {
                // Read ID3v2 version
                let file = File::open(&self.path)?;
                let mut reader = BufReader::new(file);
                let mut header = [0u8; 10];
                reader.read_exact(&mut header)?;
                if header.len() >= 4 {
                    Ok(format!("2.{}", header[3]))
                } else {
                    Ok("2.x".to_string())
                }
            }
            _ => Ok(self.file_type.clone()),
        }
    }
}

/// Metadata container
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<CoverArt>,
}

/// Cover art data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverArt {
    #[serde(serialize_with = "serialize_as_base64")]
    pub data: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ============================================================================
// PyO3 Bindings (only compiled when "python" feature is enabled)
// ============================================================================

#[cfg(feature = "python")]
#[pymodule]
fn oxidant(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAudioFile>()?;
    m.add_class::<PyMetadata>()?;
    m.add_class::<PyCoverArt>()?;
    m.add_class::<BatchProcessor>()?;
    m.add_class::<PyBatchResult>()?;
    Ok(())
}

#[cfg(feature = "python")]
#[pyclass(name = "AudioFile")]
pub struct PyAudioFile {
    #[pyo3(get)]
    path: String,
    #[pyo3(get)]
    file_type: String,
    audio: AudioFile,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyAudioFile {
    #[new]
    fn new(path: String) -> PyResult<Self> {
        let audio = AudioFile::new(path)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        let file_type = audio.file_type.clone();
        Ok(Self { path: audio.path.clone(), file_type, audio })
    }

    fn get_metadata(&self) -> PyResult<String> {
        self.audio.get_metadata()
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
    }

    fn set_metadata(&self, metadata_json: String) -> PyResult<()> {
        self.audio.set_metadata(metadata_json)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
    }

    fn get_version(&self) -> PyResult<String> {
        self.audio.get_version()
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
    }
}

#[cfg(feature = "python")]
#[pyclass(name = "Metadata")]
pub struct PyMetadata {
    #[pyo3(get, set)]
    title: Option<String>,
    #[pyo3(get, set)]
    artist: Option<String>,
    #[pyo3(get, set)]
    album: Option<String>,
    #[pyo3(get, set)]
    year: Option<String>,
    #[pyo3(get, set)]
    comment: Option<String>,
    #[pyo3(get, set)]
    track: Option<String>,
    #[pyo3(get, set)]
    genre: Option<String>,
    #[pyo3(get, set)]
    album_artist: Option<String>,
    #[pyo3(get, set)]
    composer: Option<String>,
    #[pyo3(get, set)]
    lyrics: Option<String>,
    #[pyo3(get, set)]
    cover: Option<PyCoverArt>,
}

#[cfg(feature = "python")]
#[pyclass(name = "CoverArt")]
#[derive(Clone)]
pub struct PyCoverArt {
    #[pyo3(get, set)]
    data: Vec<u8>,
    #[pyo3(get, set)]
    mime_type: Option<String>,
    #[pyo3(get, set)]
    description: Option<String>,
}

// Batch processing types (only for Python)
#[cfg(feature = "python")]
#[pyclass]
pub struct BatchProcessor {
    #[pyo3(get, set)]
    pub show_progress: bool,
}

#[cfg(feature = "python")]
#[pymethods]
impl BatchProcessor {
    #[new]
    fn new() -> Self {
        BatchProcessor {
            show_progress: true,
        }
    }

    fn read_metadata_batch(&self, file_paths: Vec<String>) -> PyResult<Vec<String>> {
        let mut results = Vec::new();
        let total = file_paths.len();

        for (index, path) in file_paths.iter().enumerate() {
            if self.show_progress {
                println!("Reading {}/{}: {}", index + 1, total, path);
            }

            match AudioFile::new(path.clone()) {
                Ok(audio) => {
                    match audio.get_metadata() {
                        Ok(metadata) => results.push(metadata),
                        Err(e) => {
                            let error_json = format!(r#"{{"error": "{}", "file": "{}"}}"#, e, path);
                            results.push(error_json);
                        }
                    }
                }
                Err(e) => {
                    let error_json = format!(r#"{{"error": "{}", "file": "{}"}}"#, e, path);
                    results.push(error_json);
                }
            }
        }

        Ok(results)
    }

    fn write_metadata_batch(&self, updates: Vec<(String, String)>) -> PyResult<Vec<PyBatchResult>> {
        let mut results = Vec::new();
        let total = updates.len();

        for (index, (path, _metadata_json)) in updates.iter().enumerate() {
            if self.show_progress {
                println!("Writing {}/{}: {}", index + 1, total, path);
            }

            let result = PyBatchResult {
                file_path: path.clone(),
                success: false,
                error_message: None,
            };

            results.push(result);
        }

        Ok(results)
    }

    fn process_directory(
        &self,
        _directory: String,
        _pattern: String,
        _operation: String,
        _metadata: Option<String>,
        py: Python,
    ) -> PyResult<Py<PyAny>> {
        let results = Vec::<PyBatchResult>::new();
        Ok(PyList::new(py, results)?.into())
    }
}

#[cfg(feature = "python")]
#[pyclass(name = "BatchResult")]
#[derive(Clone)]
pub struct PyBatchResult {
    #[pyo3(get, set)]
    pub file_path: String,
    #[pyo3(get, set)]
    pub success: bool,
    #[pyo3(get, set)]
    pub error_message: Option<String>,
}
