use pyo3::prelude::*;
use std::fs::File;
use std::io::{BufReader, Read, Seek};

mod id3;
mod flac;
mod utils;

use id3::{Id3v1Tag, Id3v2Tag};
use flac::{FlacMetadataBlock, FlacMetadataBlockType, FLAC_SIGNATURE, VorbisFields, FlacPicture};

/// Oxidant - A high-performance audio metadata library
#[pymodule]
fn oxidant(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<AudioFile>()?;
    m.add_class::<Metadata>()?;
    m.add_class::<CoverArt>()?;
    Ok(())
}

/// Audio file metadata handler
#[pyclass]
pub struct AudioFile {
    #[pyo3(get)]
    path: String,
    #[pyo3(get)]
    file_type: String,
}

#[pymethods]
impl AudioFile {
    /// Create a new AudioFile instance
    #[new]
    fn new(path: String) -> PyResult<Self> {
        let file_type = Self::detect_file_type(&path)?;
        Ok(AudioFile { path, file_type })
    }

    /// Read metadata from the audio file
    fn read_metadata(&self) -> PyResult<Metadata> {
        match self.file_type.as_str() {
            "id3v2" => self.read_id3v2_metadata(),
            "id3v1" => self.read_id3v1_metadata(),
            "flac" => self.read_flac_metadata(),
            _ => Ok(Metadata::default()),
        }
    }

    /// Detect file type
    #[staticmethod]
    fn detect_file_type(path: &str) -> PyResult<String> {
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

        // Check for ID3v1 (at end of file)
        if let Ok(Some(_)) = Id3v1Tag::read_from_file(path) {
            return Ok("id3v1".to_string());
        }

        Ok("unknown".to_string())
    }

    /// Read ID3v2 metadata
    fn read_id3v2_metadata(&self) -> PyResult<Metadata> {
        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);

        match Id3v2Tag::read(&mut reader) {
            Ok(Some(tag)) => {
                let mut metadata = Metadata::default();
                metadata.file_type = "ID3v2".to_string();
                metadata.version = format!("{}.{}", tag.header.version.0, tag.header.version.1);

                // Parse frames
                for frame in &tag.frames {
                    match frame.frame_id.as_str() {
                        "TIT2" => metadata.title = Some(self.decode_text_frame(&frame.data)),
                        "TPE1" => metadata.artist = Some(self.decode_text_frame(&frame.data)),
                        "TALB" => metadata.album = Some(self.decode_text_frame(&frame.data)),
                        "TYER" | "TDRC" => metadata.year = Some(self.decode_text_frame(&frame.data)),
                        "TRCK" => metadata.track = Some(self.decode_text_frame(&frame.data)),
                        "TCON" => metadata.genre = Some(self.decode_text_frame(&frame.data)),
                        "COMM" => metadata.comment = Some(self.decode_text_frame(&frame.data)),
                        _ => {}
                    }
                }

                Ok(metadata)
            }
            Ok(None) => Ok(Metadata::default()),
            Err(e) => Err(pyo3::exceptions::PyIOError::new_err(e.to_string())),
        }
    }

    /// Read ID3v1 metadata
    fn read_id3v1_metadata(&self) -> PyResult<Metadata> {
        match Id3v1Tag::read_from_file(&self.path) {
            Ok(Some(tag)) => {
                let mut metadata = Metadata::default();
                metadata.file_type = "ID3v1".to_string();
                metadata.version = "1.1".to_string();
                metadata.title = if !tag.title.is_empty() { Some(tag.title) } else { None };
                metadata.artist = if !tag.artist.is_empty() { Some(tag.artist) } else { None };
                metadata.album = if !tag.album.is_empty() { Some(tag.album) } else { None };
                metadata.year = if !tag.year.is_empty() { Some(tag.year) } else { None };
                metadata.comment = if !tag.comment.is_empty() { Some(tag.comment) } else { None };
                metadata.track = tag.track.map(|t| t.to_string());
                Ok(metadata)
            }
            Ok(None) => Ok(Metadata::default()),
            Err(e) => Err(pyo3::exceptions::PyIOError::new_err(e.to_string())),
        }
    }

    /// Read FLAC metadata
    fn read_flac_metadata(&self) -> PyResult<Metadata> {
        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);

        // Check FLAC signature
        let mut signature = [0u8; 4];
        reader.read_exact(&mut signature)?;

        if signature != *FLAC_SIGNATURE {
            return Ok(Metadata::default());
        }

        let mut metadata = Metadata::default();
        metadata.file_type = "FLAC".to_string();

        // Read metadata blocks
        loop {
            match FlacMetadataBlock::read(&mut reader) {
                                    Ok(block) => {
                                        if block.header.block_type == FlacMetadataBlockType::VorbisComment {
                                            if let Ok(vorbis) = flac::VorbisComment::read(&mut std::io::Cursor::new(&block.data)) {
                                                metadata.title = vorbis.get(VorbisFields::TITLE).cloned();
                                                metadata.artist = vorbis.get(VorbisFields::ARTIST).cloned();
                                                metadata.album = vorbis.get(VorbisFields::ALBUM).cloned();
                                                metadata.year = vorbis.get(VorbisFields::DATE).cloned();
                                                metadata.track = vorbis.get(VorbisFields::TRACKNUMBER).cloned();
                                                metadata.genre = vorbis.get(VorbisFields::GENRE).cloned();
                                                metadata.comment = vorbis.get(VorbisFields::COMMENT).cloned();
                                            }
                                        }
            
                                        if block.header.is_last {
                                            break;
                                        }
                                    }
                                    Err(_) => break,
                                }        }

        Ok(metadata)
    }

    /// Decode text frame data
    fn decode_text_frame(&self, data: &[u8]) -> String {
        if data.is_empty() {
            return String::new();
        }

        let encoding = utils::encoding::TextEncoding::from_byte(data[0]);
        utils::encoding::decode_text(&data[1..], encoding)
    }

    /// Extract cover art from audio file
    fn extract_cover(&self) -> PyResult<Option<CoverArt>> {
        match self.file_type.as_str() {
            "flac" => self.extract_flac_cover(),
            _ => Ok(None),
        }
    }

    /// Extract cover art from FLAC file
    fn extract_flac_cover(&self) -> PyResult<Option<CoverArt>> {
        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);

        // Check FLAC signature
        let mut signature = [0u8; 4];
        reader.read_exact(&mut signature)?;

        if signature != *FLAC_SIGNATURE {
            return Ok(None);
        }

        // Read metadata blocks
        loop {
            match FlacMetadataBlock::read(&mut reader) {
                Ok(block) => {
                    if block.header.block_type == FlacMetadataBlockType::Picture {
                        if let Ok(picture) = FlacPicture::read_from_data(&block.data) {
                            return Ok(Some(CoverArt {
                                mime_type: picture.mime_type.clone(),
                                width: picture.width,
                                height: picture.height,
                                depth: picture.depth,
                                description: picture.description,
                                data: picture.data,
                            }));
                        }
                    }

                    if block.header.is_last {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        Ok(None)
    }
}

/// Cover art structure
#[pyclass]
pub struct CoverArt {
    #[pyo3(get)]
    pub mime_type: String,
    #[pyo3(get)]
    pub width: u32,
    #[pyo3(get)]
    pub height: u32,
    #[pyo3(get)]
    pub depth: u32,
    #[pyo3(get)]
    pub description: String,
    #[pyo3(get)]
    pub data: Vec<u8>,
}

#[pymethods]
impl CoverArt {
    /// Save cover art to file
    fn save(&self, path: String) -> PyResult<()> {
        use std::io::Write;
        let mut file = File::create(path)?;
        file.write_all(&self.data)?;
        Ok(())
    }

    /// Get file extension
    fn get_extension(&self) -> String {
        match self.mime_type.as_str() {
            "image/jpeg" | "image/jpg" => "jpg".to_string(),
            "image/png" => "png".to_string(),
            "image/gif" => "gif".to_string(),
            "image/webp" => "webp".to_string(),
            "image/bmp" => "bmp".to_string(),
            "image/tiff" => "tiff".to_string(),
            _ => "jpg".to_string(),
        }
    }

    /// String representation
    fn __str__(&self) -> String {
        format!(
            "CoverArt(mime_type={}, {}x{}, depth={})",
            self.mime_type, self.width, self.height, self.depth
        )
    }

    /// Representation
    fn __repr__(&self) -> String {
        self.__str__()
    }
}

/// Audio metadata structure
#[pyclass]
#[derive(Default)]
pub struct Metadata {
    #[pyo3(get)]
    pub file_type: String,
    #[pyo3(get)]
    pub version: String,
    #[pyo3(get, set)]
    pub title: Option<String>,
    #[pyo3(get, set)]
    pub artist: Option<String>,
    #[pyo3(get, set)]
    pub album: Option<String>,
    #[pyo3(get, set)]
    pub year: Option<String>,
    #[pyo3(get, set)]
    pub track: Option<String>,
    #[pyo3(get, set)]
    pub genre: Option<String>,
    #[pyo3(get, set)]
    pub comment: Option<String>,
}

#[pymethods]
impl Metadata {
    /// Create a new Metadata instance
    #[new]
    fn new() -> Self {
        Metadata::default()
    }

    /// Convert to dictionary
    fn to_dict(&self) -> PyResult<pyo3::Py<pyo3::types::PyDict>> {
        Python::with_gil(|py| {
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("file_type", &self.file_type)?;
            dict.set_item("version", &self.version)?;
            dict.set_item("title", self.title.as_ref())?;
            dict.set_item("artist", self.artist.as_ref())?;
            dict.set_item("album", self.album.as_ref())?;
            dict.set_item("year", self.year.as_ref())?;
            dict.set_item("track", self.track.as_ref())?;
            dict.set_item("genre", self.genre.as_ref())?;
            dict.set_item("comment", self.comment.as_ref())?;
            Ok(dict.into())
        })
    }

    /// String representation
    fn __str__(&self) -> String {
        format!(
            "Metadata(file_type={}, version={}, title={}, artist={}, album={})",
            self.file_type,
            self.version,
            self.title.as_deref().unwrap_or("None"),
            self.artist.as_deref().unwrap_or("None"),
            self.album.as_deref().unwrap_or("None")
        )
    }

    /// Representation
    fn __repr__(&self) -> String {
        self.__str__()
    }
}
