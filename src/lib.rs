use pyo3::prelude::*;
use pyo3::Bound;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use serde::{Serialize, Deserialize};

mod id3;
mod flac;
mod utils;

use id3::{Id3v1Tag, Id3v2Tag};
use flac::{FlacMetadataBlock, FlacMetadataBlockType, FLAC_SIGNATURE, VorbisFields, FlacPicture};

/// Oxidant - A high-performance audio metadata library
#[pymodule]
fn oxidant(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
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

// Private implementation block for internal methods
impl AudioFile {
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
                                                metadata.lyrics = vorbis.get(VorbisFields::LYRICS).cloned();
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
            "id3v2" => self.extract_id3v2_cover(),
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

    /// Extract cover art from ID3v2 file
    fn extract_id3v2_cover(&self) -> PyResult<Option<CoverArt>> {
        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);

        match Id3v2Tag::read(&mut reader) {
            Ok(Some(tag)) => {
                for frame in &tag.frames {
                    if frame.frame_id == "APIC" {
                        if let Some((mime_type, _picture_type, description, data)) = id3::frames::decode_apic_frame(&frame.data) {
                            return Ok(Some(CoverArt {
                                mime_type,
                                width: 0,  // ID3v2 doesn't store dimensions
                                height: 0,
                                depth: 0,
                                description,
                                data,
                            }));
                        }
                    }
                }
                Ok(None)
            }
            Ok(None) => Ok(None),
            Err(e) => Err(pyo3::exceptions::PyIOError::new_err(e.to_string())),
        }
    }

    /// Set cover art for audio file
    fn set_cover(&self, image_path: String, mime_type: String, description: String) -> PyResult<()> {
        match self.file_type.as_str() {
            "flac" => self.set_flac_cover(image_path, mime_type, description),
            "id3v2" => self.set_id3v2_cover(image_path, mime_type, description),
            _ => Err(pyo3::exceptions::PyValueError::new_err(
                format!("File type {} does not support cover art modification", self.file_type)
            )),
        }
    }

    /// Set cover art for FLAC file
    fn set_flac_cover(&self, image_path: String, mime_type: String, description: String) -> PyResult<()> {
        // Read image data
        let image_data = std::fs::read(&image_path)?;

        // Create new picture
        let new_picture = flac::FlacPicture::new(image_data, mime_type, description);
        let picture_data = new_picture.to_bytes();

        // Read the whole file
        let mut file_data = std::fs::read(&self.path)?;

        // Find and replace the first PICTURE block
        let mut pos = 4; // Skip FLAC signature
        let mut found_picture = false;

        while pos < file_data.len() {
            if pos + 4 > file_data.len() {
                break;
            }

            // Read block header
            let is_last = (file_data[pos] & 0x80) != 0;
            let block_type = file_data[pos] & 0x7F;

            if block_type == 6 { // Picture block type
                // Read block length
                let block_length = (((file_data[pos + 1] as u32) << 16) |
                                  ((file_data[pos + 2] as u32) << 8) |
                                  (file_data[pos + 3] as u32)) as usize;

                            let header_size = 4;
                            let total_size = header_size + block_length;
                            let new_block_length = picture_data.len();
                            let total_size_usize = total_size;
                // Create new block header
                let mut new_header = [0u8; 4];
                new_header[0] = if is_last { 0x80 | 6 } else { 6 };
                new_header[1] = ((new_block_length >> 16) & 0xFF) as u8;
                new_header[2] = ((new_block_length >> 8) & 0xFF) as u8;
                new_header[3] = (new_block_length & 0xFF) as u8;

                // Replace the block
                let mut new_file_data = Vec::new();
                new_file_data.extend_from_slice(&file_data[..pos]);
                new_file_data.extend_from_slice(&new_header);
                new_file_data.extend_from_slice(&picture_data);
                new_file_data.extend_from_slice(&file_data[pos + total_size_usize..]);

                file_data = new_file_data;
                found_picture = true;
                break;
            } else {
                // Move to next block
                let block_length: usize = (((file_data[pos + 1] as u32) << 16) |
                                          ((file_data[pos + 2] as u32) << 8) |
                                          (file_data[pos + 3] as u32)) as usize;
                pos += 4 + block_length;

                if is_last {
                    break;
                }
            }
        }

        // If no picture block found, insert a new one before the audio data
        if !found_picture {
            // Find the position before audio data (after last metadata block)
            let insert_pos = pos;

            // Create new picture block
            let mut new_header = [0u8; 4];
            let new_block_length = picture_data.len();
            new_header[0] = 0x80 | 6; // Last block + Picture type
            new_header[1] = ((new_block_length >> 16) & 0xFF) as u8;
            new_header[2] = ((new_block_length >> 8) & 0xFF) as u8;
            new_header[3] = (new_block_length & 0xFF) as u8;

            // Update the previous block's last flag
            if insert_pos > 4 {
                file_data[insert_pos - 4] &= 0x7F; // Clear last flag
            }

            // Insert new block
            let mut new_file_data = Vec::new();
            new_file_data.extend_from_slice(&file_data[..insert_pos]);
            new_file_data.extend_from_slice(&new_header);
            new_file_data.extend_from_slice(&picture_data);
            new_file_data.extend_from_slice(&file_data[insert_pos..]);

            file_data = new_file_data;
        }

        // Write modified file
        std::fs::write(&self.path, file_data)?;

        Ok(())
    }

    /// Set cover art for ID3v2 file
    fn set_id3v2_cover(&self, image_path: String, mime_type: String, description: String) -> PyResult<()> {
        use id3::frames::{encode_apic_frame, PictureType};

        // Read image data
        let image_data = std::fs::read(&image_path)?;

        // Create APIC frame
        let apic_data = encode_apic_frame(&mime_type, PictureType::CoverFront, &description, &image_data);

        // Read the whole file
        let mut file_data = std::fs::read(&self.path)?;

        // Check for ID3v2 tag
        if file_data.len() < 10 || &file_data[0..3] != b"ID3" {
            return Err(pyo3::exceptions::PyValueError::new_err("Not a valid ID3v2 file"));
        }

        // Get ID3v2 header info
        let version = (file_data[3], file_data[4]);
        let _flags = file_data[5];
        let tag_size: usize = (((file_data[6] as u32) << 21) |
                      ((file_data[7] as u32) << 14) |
                      ((file_data[8] as u32) << 7) |
                      (file_data[9] as u32)) as usize;

        let header_size: usize = 10;
        let tag_end: usize = header_size + tag_size;

        // Find and replace existing APIC frames
        let mut pos: usize = header_size;
        let mut frames_before_apic: Vec<(String, Vec<u8>)> = Vec::new();

        while pos < tag_end {
            if pos + 10 > file_data.len() {
                break;
            }

            // Read frame header
            let frame_id = String::from_utf8_lossy(&file_data[pos..pos + 4]).to_string();

            // Check for padding (all zeros)
            if frame_id.chars().all(|c| c == '\0') {
                // Padding found, stop reading frames
                break;
            }

            // Read frame size
            let frame_size: usize = if version.0 >= 4 {
                // ID3v2.4 uses synchsafe integers
                (((file_data[pos + 4] as u32) << 21) |
                ((file_data[pos + 5] as u32) << 14) |
                ((file_data[pos + 6] as u32) << 7) |
                (file_data[pos + 7] as u32)) as usize
            } else {
                // ID3v2.3 uses regular integers
                (((file_data[pos + 4] as u32) << 24) |
                ((file_data[pos + 5] as u32) << 16) |
                ((file_data[pos + 6] as u32) << 8) |
                (file_data[pos + 7] as u32)) as usize
            };

            let frame_header_size: usize = 10;
            let frame_end = pos + frame_header_size + frame_size;

            if frame_end > file_data.len() {
                break;
            }

            let frame_data = file_data[pos + frame_header_size..frame_end].to_vec();

            if frame_id != "APIC" {
                frames_before_apic.push((frame_id, frame_data));
            }

            pos += frame_header_size + frame_size;
        }

        // Create new APIC frame
        let new_apic_frame = create_id3v2_frame("APIC", &apic_data, version.0);

        // Build new tag data
        let mut new_tag_data = Vec::new();

        // Add frames before APIC
        for (frame_id, frame_data) in frames_before_apic {
            new_tag_data.extend_from_slice(&create_id3v2_frame(&frame_id, &frame_data, version.0));
        }

        // Add new APIC frame
        new_tag_data.extend_from_slice(&new_apic_frame);

        // Update ID3v2 header with new size
        let new_tag_size = new_tag_data.len();

        // Calculate synchsafe size (each byte has only 7 bits)
        let synchsafe_size = to_synchsafe(new_tag_size);

        file_data[6] = ((synchsafe_size >> 21) & 0x7F) as u8;
        file_data[7] = ((synchsafe_size >> 14) & 0x7F) as u8;
        file_data[8] = ((synchsafe_size >> 7) & 0x7F) as u8;
        file_data[9] = (synchsafe_size & 0x7F) as u8;

        // Build new file data
        let mut new_file_data = Vec::new();
        new_file_data.extend_from_slice(&file_data[..header_size]);
        new_file_data.extend_from_slice(&new_tag_data);
        new_file_data.extend_from_slice(&file_data[tag_end..]);

        // Write modified file
        std::fs::write(&self.path, new_file_data)?;

        Ok(())
    }

    /// Get lyrics from audio file
    fn get_lyrics(&self) -> PyResult<Option<String>> {
        match self.file_type.as_str() {
            "flac" => self.get_flac_lyrics(),
            "id3v2" => self.get_id3v2_lyrics(),
            _ => Ok(None),
        }
    }

    /// Get lyrics from FLAC file
    fn get_flac_lyrics(&self) -> PyResult<Option<String>> {
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
                    if block.header.block_type == FlacMetadataBlockType::VorbisComment {
                        if let Ok(vorbis) = flac::VorbisComment::read(&mut std::io::Cursor::new(&block.data)) {
                            if let Some(lyrics) = vorbis.get(VorbisFields::LYRICS) {
                                return Ok(Some(lyrics.clone()));
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

        Ok(None)
    }

    /// Get lyrics from ID3v2 file
    fn get_id3v2_lyrics(&self) -> PyResult<Option<String>> {
        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);

        match Id3v2Tag::read(&mut reader) {
            Ok(Some(tag)) => {
                for frame in &tag.frames {
                    if frame.frame_id == "USLT" {
                        if let Some((_language, _description, lyrics)) = id3::frames::decode_uslt_frame(&frame.data) {
                            return Ok(Some(lyrics));
                        }
                    }
                }
                Ok(None)
            }
            Ok(None) => Ok(None),
            Err(e) => Err(pyo3::exceptions::PyIOError::new_err(e.to_string())),
        }
    }

    /// Set lyrics for audio file
    fn set_lyrics(&self, lyrics: String) -> PyResult<()> {
        match self.file_type.as_str() {
            "flac" => self.set_flac_lyrics(lyrics),
            "id3v2" => self.set_id3v2_lyrics(lyrics),
            _ => Err(pyo3::exceptions::PyValueError::new_err(
                format!("File type {} does not support lyrics modification", self.file_type)
            )),
        }
    }

    /// Set lyrics for FLAC file
    fn set_flac_lyrics(&self, lyrics: String) -> PyResult<()> {
        // Read the whole file
        let mut file_data = std::fs::read(&self.path)?;

        // Find Vorbis Comment block
        let mut pos = 4; // Skip FLAC signature
        let mut found_vorbis = false;

        while pos < file_data.len() {
            if pos + 4 > file_data.len() {
                break;
            }

            // Read block header
            let is_last = (file_data[pos] & 0x80) != 0;
            let block_type = file_data[pos] & 0x7F;

            if block_type == 4 { // Vorbis Comment block type
                // Read block length
                let block_length = (((file_data[pos + 1] as u32) << 16) |
                                  ((file_data[pos + 2] as u32) << 8) |
                                  (file_data[pos + 3] as u32)) as usize;

                let header_size = 4;
                let total_size = header_size + block_length;

                // Read existing Vorbis comment
                let vorbis_data = &file_data[pos + header_size..pos + total_size];
                if let Ok(mut vorbis) = flac::VorbisComment::read(&mut std::io::Cursor::new(vorbis_data)) {
                    // Set lyrics
                    vorbis.set(VorbisFields::LYRICS, &lyrics);
                    let new_vorbis_data = vorbis.to_bytes();

                    // Update block
                    let new_block_length = new_vorbis_data.len();
                    let mut new_header = [0u8; 4];
                    new_header[0] = if is_last { 0x80 | 4 } else { 4 };
                    new_header[1] = ((new_block_length >> 16) & 0xFF) as u8;
                    new_header[2] = ((new_block_length >> 8) & 0xFF) as u8;
                    new_header[3] = (new_block_length & 0xFF) as u8;

                    // Replace the block
                    let mut new_file_data = Vec::new();
                    new_file_data.extend_from_slice(&file_data[..pos]);
                    new_file_data.extend_from_slice(&new_header);
                    new_file_data.extend_from_slice(&new_vorbis_data);
                    new_file_data.extend_from_slice(&file_data[pos + total_size..]);

                    file_data = new_file_data;
                    found_vorbis = true;
                    break;
                }
            } else {
                // Move to next block
                let block_length: usize = (((file_data[pos + 1] as u32) << 16) |
                                          ((file_data[pos + 2] as u32) << 8) |
                                          (file_data[pos + 3] as u32)) as usize;
                pos += 4 + block_length;

                if is_last {
                    break;
                }
            }
        }

        // If no Vorbis Comment block found, create a new one
        if !found_vorbis {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "No Vorbis Comment block found in FLAC file"
            ));
        }

        // Write modified file
        std::fs::write(&self.path, file_data)?;

        Ok(())
    }

    /// Set lyrics for ID3v2 file
    fn set_id3v2_lyrics(&self, lyrics: String) -> PyResult<()> {
        use id3::frames::encode_uslt_frame;

        // Create USLT frame (language: "eng", description: "")
        let uslt_data = encode_uslt_frame("eng", "", &lyrics);

        // Read the whole file
        let mut file_data = std::fs::read(&self.path)?;

        // Check for ID3v2 tag
        if file_data.len() < 10 || &file_data[0..3] != b"ID3" {
            return Err(pyo3::exceptions::PyValueError::new_err("Not a valid ID3v2 file"));
        }

        // Get ID3v2 header info
        let version = (file_data[3], file_data[4]);
        let tag_size: usize = (((file_data[6] as u32) << 21) |
                      ((file_data[7] as u32) << 14) |
                      ((file_data[8] as u32) << 7) |
                      (file_data[9] as u32)) as usize;

        let header_size: usize = 10;
        let tag_end: usize = header_size + tag_size;

        // Find and replace existing USLT frames
        let mut pos: usize = header_size;
        let mut frames_before_uslt: Vec<(String, Vec<u8>)> = Vec::new();

        while pos < tag_end {
            if pos + 10 > file_data.len() {
                break;
            }

            // Read frame header
            let frame_id = String::from_utf8_lossy(&file_data[pos..pos + 4]).to_string();

            // Check for padding (all zeros)
            if frame_id.chars().all(|c| c == '\0') {
                // Padding found, stop reading frames
                break;
            }

            // Read frame size
            let frame_size: usize = if version.0 >= 4 {
                // ID3v2.4 uses synchsafe integers
                (((file_data[pos + 4] as u32) << 21) |
                ((file_data[pos + 5] as u32) << 14) |
                ((file_data[pos + 6] as u32) << 7) |
                (file_data[pos + 7] as u32)) as usize
            } else {
                // ID3v2.3 uses regular integers
                (((file_data[pos + 4] as u32) << 24) |
                ((file_data[pos + 5] as u32) << 16) |
                ((file_data[pos + 6] as u32) << 8) |
                (file_data[pos + 7] as u32)) as usize
            };

            let frame_header_size: usize = 10;
            let frame_end = pos + frame_header_size + frame_size;

            if frame_end > file_data.len() {
                break;
            }

            let frame_data = file_data[pos + frame_header_size..frame_end].to_vec();

            if frame_id != "USLT" {
                frames_before_uslt.push((frame_id, frame_data));
            }

            pos += frame_header_size + frame_size;
        }

        // Create new USLT frame
        let new_uslt_frame = create_id3v2_frame("USLT", &uslt_data, version.0);

        // Build new tag data
        let mut new_tag_data = Vec::new();

        // Add frames before USLT
        for (frame_id, frame_data) in frames_before_uslt {
            new_tag_data.extend_from_slice(&create_id3v2_frame(&frame_id, &frame_data, version.0));
        }

        // Add new USLT frame
        new_tag_data.extend_from_slice(&new_uslt_frame);

        // Update ID3v2 header with new size
        let new_tag_size = new_tag_data.len();

        // Calculate synchsafe size (each byte has only 7 bits)
        let synchsafe_size = to_synchsafe(new_tag_size);

        file_data[6] = ((synchsafe_size >> 21) & 0x7F) as u8;
        file_data[7] = ((synchsafe_size >> 14) & 0x7F) as u8;
        file_data[8] = ((synchsafe_size >> 7) & 0x7F) as u8;
        file_data[9] = (synchsafe_size & 0x7F) as u8;

        // Build new file data
        let mut new_file_data = Vec::new();
        new_file_data.extend_from_slice(&file_data[..header_size]);
        new_file_data.extend_from_slice(&new_tag_data);
        new_file_data.extend_from_slice(&file_data[tag_end..]);

        // Write modified file
        std::fs::write(&self.path, new_file_data)?;

        Ok(())
    }

    /// Remove lyrics from audio file
    fn remove_lyrics(&self) -> PyResult<()> {
        match self.file_type.as_str() {
            "flac" => self.remove_flac_lyrics(),
            "id3v2" => self.remove_id3v2_lyrics(),
            _ => Err(pyo3::exceptions::PyValueError::new_err(
                format!("File type {} does not support lyrics modification", self.file_type)
            )),
        }
    }

    /// Remove lyrics from FLAC file
    fn remove_flac_lyrics(&self) -> PyResult<()> {
        // Read the whole file
        let mut file_data = std::fs::read(&self.path)?;

        // Find Vorbis Comment block
        let mut pos = 4; // Skip FLAC signature
        let mut found_vorbis = false;

        while pos < file_data.len() {
            if pos + 4 > file_data.len() {
                break;
            }

            // Read block header
            let is_last = (file_data[pos] & 0x80) != 0;
            let block_type = file_data[pos] & 0x7F;

            if block_type == 4 { // Vorbis Comment block type
                // Read block length
                let block_length = (((file_data[pos + 1] as u32) << 16) |
                                  ((file_data[pos + 2] as u32) << 8) |
                                  (file_data[pos + 3] as u32)) as usize;

                let header_size = 4;
                let total_size = header_size + block_length;

                // Read existing Vorbis comment
                let vorbis_data = &file_data[pos + header_size..pos + total_size];
                if let Ok(mut vorbis) = flac::VorbisComment::read(&mut std::io::Cursor::new(vorbis_data)) {
                    // Remove lyrics
                    vorbis.remove(VorbisFields::LYRICS);
                    let new_vorbis_data = vorbis.to_bytes();

                    // Update block
                    let new_block_length = new_vorbis_data.len();
                    let mut new_header = [0u8; 4];
                    new_header[0] = if is_last { 0x80 | 4 } else { 4 };
                    new_header[1] = ((new_block_length >> 16) & 0xFF) as u8;
                    new_header[2] = ((new_block_length >> 8) & 0xFF) as u8;
                    new_header[3] = (new_block_length & 0xFF) as u8;

                    // Replace the block
                    let mut new_file_data = Vec::new();
                    new_file_data.extend_from_slice(&file_data[..pos]);
                    new_file_data.extend_from_slice(&new_header);
                    new_file_data.extend_from_slice(&new_vorbis_data);
                    new_file_data.extend_from_slice(&file_data[pos + total_size..]);

                    file_data = new_file_data;
                    found_vorbis = true;
                    break;
                }
            } else {
                // Move to next block
                let block_length: usize = (((file_data[pos + 1] as u32) << 16) |
                                          ((file_data[pos + 2] as u32) << 8) |
                                          (file_data[pos + 3] as u32)) as usize;
                pos += 4 + block_length;

                if is_last {
                    break;
                }
            }
        }

        // If no Vorbis Comment block found, create a new one
        if !found_vorbis {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "No Vorbis Comment block found in FLAC file"
            ));
        }

        // Write modified file
        std::fs::write(&self.path, file_data)?;

        Ok(())
    }

    /// Remove lyrics from ID3v2 file
    fn remove_id3v2_lyrics(&self) -> PyResult<()> {
        // Read the whole file
        let mut file_data = std::fs::read(&self.path)?;

        // Check for ID3v2 tag
        if file_data.len() < 10 || &file_data[0..3] != b"ID3" {
            return Err(pyo3::exceptions::PyValueError::new_err("Not a valid ID3v2 file"));
        }

        // Get ID3v2 header info
        let version = (file_data[3], file_data[4]);
        let tag_size: usize = (((file_data[6] as u32) << 21) |
                      ((file_data[7] as u32) << 14) |
                      ((file_data[8] as u32) << 7) |
                      (file_data[9] as u32)) as usize;

        let header_size: usize = 10;
        let tag_end: usize = header_size + tag_size;

        // Find and remove existing USLT frames
        let mut pos: usize = header_size;
        let mut frames: Vec<(String, Vec<u8>)> = Vec::new();

        while pos < tag_end {
            if pos + 10 > file_data.len() {
                break;
            }

            // Read frame header
            let frame_id = String::from_utf8_lossy(&file_data[pos..pos + 4]).to_string();

            // Check for padding (all zeros)
            if frame_id.chars().all(|c| c == '\0') {
                // Padding found, stop reading frames
                break;
            }

            // Read frame size
            let frame_size: usize = if version.0 >= 4 {
                // ID3v2.4 uses synchsafe integers
                (((file_data[pos + 4] as u32) << 21) |
                ((file_data[pos + 5] as u32) << 14) |
                ((file_data[pos + 6] as u32) << 7) |
                (file_data[pos + 7] as u32)) as usize
            } else {
                // ID3v2.3 uses regular integers
                (((file_data[pos + 4] as u32) << 24) |
                ((file_data[pos + 5] as u32) << 16) |
                ((file_data[pos + 6] as u32) << 8) |
                (file_data[pos + 7] as u32)) as usize
            };

            let frame_header_size: usize = 10;
            let frame_end = pos + frame_header_size + frame_size;

            if frame_end > file_data.len() {
                break;
            }

            let frame_data = file_data[pos + frame_header_size..frame_end].to_vec();

            // Keep all frames except USLT
            if frame_id != "USLT" {
                frames.push((frame_id, frame_data));
            }

            pos += frame_header_size + frame_size;
        }

        // Build new tag data
        let mut new_tag_data = Vec::new();

        // Add all frames except USLT
        for (frame_id, frame_data) in frames {
            new_tag_data.extend_from_slice(&create_id3v2_frame(&frame_id, &frame_data, version.0));
        }

        // Update ID3v2 header with new size
        let new_tag_size = new_tag_data.len();

        // Calculate synchsafe size (each byte has only 7 bits)
        let synchsafe_size = to_synchsafe(new_tag_size);

        file_data[6] = ((synchsafe_size >> 21) & 0x7F) as u8;
        file_data[7] = ((synchsafe_size >> 14) & 0x7F) as u8;
        file_data[8] = ((synchsafe_size >> 7) & 0x7F) as u8;
        file_data[9] = (synchsafe_size & 0x7F) as u8;

        // Build new file data
        let mut new_file_data = Vec::new();
        new_file_data.extend_from_slice(&file_data[..header_size]);
        new_file_data.extend_from_slice(&new_tag_data);
        new_file_data.extend_from_slice(&file_data[tag_end..]);

        // Write modified file
        std::fs::write(&self.path, new_file_data)?;

        Ok(())
    }

    /// Set title for FLAC file
    fn set_flac_title(&self, title: String) -> PyResult<()> {
        self.set_flac_vorbis_field(VorbisFields::TITLE, &title)
    }

    /// Set artist for FLAC file
    fn set_flac_artist(&self, artist: String) -> PyResult<()> {
        self.set_flac_vorbis_field(VorbisFields::ARTIST, &artist)
    }

    /// Set album for FLAC file
    fn set_flac_album(&self, album: String) -> PyResult<()> {
        self.set_flac_vorbis_field(VorbisFields::ALBUM, &album)
    }

    /// Set year for FLAC file
    fn set_flac_year(&self, year: String) -> PyResult<()> {
        self.set_flac_vorbis_field(VorbisFields::DATE, &year)
    }

    /// Set track number for FLAC file
    fn set_flac_track(&self, track: String) -> PyResult<()> {
        self.set_flac_vorbis_field(VorbisFields::TRACKNUMBER, &track)
    }

    /// Set genre for FLAC file
    fn set_flac_genre(&self, genre: String) -> PyResult<()> {
        self.set_flac_vorbis_field(VorbisFields::GENRE, &genre)
    }

    /// Set comment for FLAC file
    fn set_flac_comment(&self, comment: String) -> PyResult<()> {
        self.set_flac_vorbis_field(VorbisFields::COMMENT, &comment)
    }

    /// Helper method to set a Vorbis comment field in FLAC file
    fn set_flac_vorbis_field(&self, field: &str, value: &str) -> PyResult<()> {
        // Read the whole file
        let mut file_data = std::fs::read(&self.path)?;

        // Find Vorbis Comment block
        let mut pos = 4; // Skip FLAC signature
        let mut found_vorbis = false;

        while pos < file_data.len() {
            if pos + 4 > file_data.len() {
                break;
            }

            // Read block header
            let is_last = (file_data[pos] & 0x80) != 0;
            let block_type = file_data[pos] & 0x7F;

            if block_type == 4 { // Vorbis Comment block type
                // Read block length
                let block_length = (((file_data[pos + 1] as u32) << 16) |
                                  ((file_data[pos + 2] as u32) << 8) |
                                  (file_data[pos + 3] as u32)) as usize;

                let header_size = 4;
                let total_size = header_size + block_length;

                // Read existing Vorbis comment
                let vorbis_data = &file_data[pos + header_size..pos + total_size];
                if let Ok(mut vorbis) = flac::VorbisComment::read(&mut std::io::Cursor::new(vorbis_data)) {
                    // Set the field
                    vorbis.set(field, value);
                    let new_vorbis_data = vorbis.to_bytes();

                    // Update block
                    let new_block_length = new_vorbis_data.len();
                    let mut new_header = [0u8; 4];
                    new_header[0] = if is_last { 0x80 | 4 } else { 4 };
                    new_header[1] = ((new_block_length >> 16) & 0xFF) as u8;
                    new_header[2] = ((new_block_length >> 8) & 0xFF) as u8;
                    new_header[3] = (new_block_length & 0xFF) as u8;

                    // Replace the block
                    let mut new_file_data = Vec::new();
                    new_file_data.extend_from_slice(&file_data[..pos]);
                    new_file_data.extend_from_slice(&new_header);
                    new_file_data.extend_from_slice(&new_vorbis_data);
                    new_file_data.extend_from_slice(&file_data[pos + total_size..]);

                    file_data = new_file_data;
                    found_vorbis = true;
                    break;
                }
            } else {
                // Move to next block
                let block_length: usize = (((file_data[pos + 1] as u32) << 16) |
                                          ((file_data[pos + 2] as u32) << 8) |
                                          (file_data[pos + 3] as u32)) as usize;
                pos += 4 + block_length;

                if is_last {
                    break;
                }
            }
        }

        // If no Vorbis Comment block found, create a new one
        if !found_vorbis {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "No Vorbis Comment block found in FLAC file"
            ));
        }

        // Write modified file
        std::fs::write(&self.path, file_data)?;

        Ok(())
    }

    /// Write all metadata to ID3v2 file
    fn write_id3v2_metadata(&self, metadata: Metadata) -> PyResult<()> {
        use id3::frames::{encode_text_frame, encode_uslt_frame, TextEncoding};

        // Read the whole file
        let mut file_data = std::fs::read(&self.path)?;

        // Check for ID3v2 tag
        if file_data.len() < 10 || &file_data[0..3] != b"ID3" {
            return Err(pyo3::exceptions::PyValueError::new_err("Not a valid ID3v2 file"));
        }

        // Get ID3v2 header info
        let version = (file_data[3], file_data[4]);
        let tag_size: usize = (((file_data[6] as u32) << 21) |
                      ((file_data[7] as u32) << 14) |
                      ((file_data[8] as u32) << 7) |
                      (file_data[9] as u32)) as usize;

        let header_size: usize = 10;
        let tag_end: usize = header_size + tag_size;

        // Read existing frames, skip ones we'll update
        let mut pos: usize = header_size;
        let mut existing_frames: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();

        while pos < tag_end {
            if pos + 10 > file_data.len() {
                break;
            }

            // Read frame header
            let frame_id = String::from_utf8_lossy(&file_data[pos..pos + 4]).to_string();

            // Check for padding
            if frame_id.chars().all(|c| c == '\0') {
                break;
            }

            // Read frame size
            let frame_size: usize = if version.0 >= 4 {
                (((file_data[pos + 4] as u32) << 21) |
                ((file_data[pos + 5] as u32) << 14) |
                ((file_data[pos + 6] as u32) << 7) |
                (file_data[pos + 7] as u32)) as usize
            } else {
                (((file_data[pos + 4] as u32) << 24) |
                ((file_data[pos + 5] as u32) << 16) |
                ((file_data[pos + 6] as u32) << 8) |
                (file_data[pos + 7] as u32)) as usize
            };

            let frame_header_size: usize = 10;
            let frame_end = pos + frame_header_size + frame_size;

            if frame_end > file_data.len() {
                break;
            }

            let frame_data = file_data[pos + frame_header_size..frame_end].to_vec();

            // Store frame if we're not updating it
            let should_keep = match frame_id.as_str() {
                "TIT2" | "TPE1" | "TALB" | "TYER" | "TDRC" | "TRCK" | "TCON" | "COMM" | "USLT" => false,
                _ => true,
            };

            if should_keep {
                existing_frames.insert(frame_id, frame_data);
            }

            pos += frame_header_size + frame_size;
        }

        // Build new tag data
        let mut new_tag_data = Vec::new();

        // Add existing non-metadata frames first
        for (frame_id, frame_data) in &existing_frames {
            if !["APIC", "USLT"].contains(&frame_id.as_str()) {
                new_tag_data.extend_from_slice(&create_id3v2_frame(frame_id, frame_data, version.0));
            }
        }

        // Add text metadata frames
        let encoding = TextEncoding::Utf8;

        if let Some(title) = &metadata.title {
            let frame_data = encode_text_frame(title, encoding);
            new_tag_data.extend_from_slice(&create_id3v2_frame("TIT2", &frame_data, version.0));
        }
        if let Some(artist) = &metadata.artist {
            let frame_data = encode_text_frame(artist, encoding);
            new_tag_data.extend_from_slice(&create_id3v2_frame("TPE1", &frame_data, version.0));
        }
        if let Some(album) = &metadata.album {
            let frame_data = encode_text_frame(album, encoding);
            new_tag_data.extend_from_slice(&create_id3v2_frame("TALB", &frame_data, version.0));
        }
        if let Some(year) = &metadata.year {
            let frame_data = encode_text_frame(year, encoding);
            // Prefer TYER for v2.3, TDRC for v2.4
            let frame_id = if version.0 >= 4 { "TDRC" } else { "TYER" };
            new_tag_data.extend_from_slice(&create_id3v2_frame(frame_id, &frame_data, version.0));
        }
        if let Some(track) = &metadata.track {
            let frame_data = encode_text_frame(track, encoding);
            new_tag_data.extend_from_slice(&create_id3v2_frame("TRCK", &frame_data, version.0));
        }
        if let Some(genre) = &metadata.genre {
            let frame_data = encode_text_frame(genre, encoding);
            new_tag_data.extend_from_slice(&create_id3v2_frame("TCON", &frame_data, version.0));
        }
        if let Some(comment) = &metadata.comment {
            let frame_data = encode_text_frame(comment, encoding);
            new_tag_data.extend_from_slice(&create_id3v2_frame("COMM", &frame_data, version.0));
        }
        if let Some(lyrics) = &metadata.lyrics {
            let frame_data = encode_uslt_frame("eng", "", lyrics);
            new_tag_data.extend_from_slice(&create_id3v2_frame("USLT", &frame_data, version.0));
        }

        // Add back APIC frames (cover art)
        if let Some(apic_data) = existing_frames.get("APIC") {
            new_tag_data.extend_from_slice(&create_id3v2_frame("APIC", apic_data, version.0));
        }

        // Update ID3v2 header with new size
        let new_tag_size = new_tag_data.len();
        let synchsafe_size = to_synchsafe(new_tag_size);

        file_data[6] = ((synchsafe_size >> 21) & 0x7F) as u8;
        file_data[7] = ((synchsafe_size >> 14) & 0x7F) as u8;
        file_data[8] = ((synchsafe_size >> 7) & 0x7F) as u8;
        file_data[9] = (synchsafe_size & 0x7F) as u8;

        // Build new file data
        let mut new_file_data = Vec::new();
        new_file_data.extend_from_slice(&file_data[..header_size]);
        new_file_data.extend_from_slice(&new_tag_data);
        new_file_data.extend_from_slice(&file_data[tag_end..]);

        // Write modified file
        std::fs::write(&self.path, new_file_data)?;

        Ok(())
    }

    /// Write all metadata to ID3v1 file
    fn write_id3v1_metadata(&self, metadata: Metadata) -> PyResult<()> {
        use encoding_rs::WINDOWS_1252;

        // Read the whole file
        let mut file_data = std::fs::read(&self.path)?;

        // Prepare ID3v1 tag (128 bytes)
        let mut tag = vec![0u8; 128];

        // Tag identifier
        tag[0..3].copy_from_slice(b"TAG");

        // Title (30 bytes)
        if let Some(title) = &metadata.title {
            let title_bytes = WINDOWS_1252.encode(title).0;
            let len = title_bytes.len().min(30);
            tag[3..3 + len].copy_from_slice(&title_bytes[..len]);
        }

        // Artist (30 bytes)
        if let Some(artist) = &metadata.artist {
            let artist_bytes = WINDOWS_1252.encode(artist).0;
            let len = artist_bytes.len().min(30);
            tag[33..33 + len].copy_from_slice(&artist_bytes[..len]);
        }

        // Album (30 bytes)
        if let Some(album) = &metadata.album {
            let album_bytes = WINDOWS_1252.encode(album).0;
            let len = album_bytes.len().min(30);
            tag[63..63 + len].copy_from_slice(&album_bytes[..len]);
        }

        // Year (4 bytes)
        if let Some(year) = &metadata.year {
            let year_bytes = year.as_bytes();
            let len = year_bytes.len().min(4);
            tag[93..93 + len].copy_from_slice(&year_bytes[..len]);
        }

        // Comment (28 or 30 bytes depending on track number presence)
        let comment_start = 97;
        let comment_len = if metadata.track.is_some() { 28 } else { 30 };

        if let Some(comment) = &metadata.comment {
            let comment_bytes = WINDOWS_1252.encode(comment).0;
            let len = comment_bytes.len().min(comment_len);
            tag[comment_start..comment_start + len].copy_from_slice(&comment_bytes[..len]);
        }

        // Track number (if present)
        if let Some(track) = &metadata.track {
            if let Ok(track_num) = track.parse::<u8>() {
                tag[125] = 0;
                tag[126] = track_num;
            }
        }

        // Genre (ID3v1.1 uses standard genres, but we'll skip for now)
        // tag[127] = 0;

        // Check if file already has ID3v1 tag
        let file_len = file_data.len();
        if file_len >= 128 && &file_data[file_len - 128..file_len - 125] == b"TAG" {
            // Replace existing tag
            file_data[file_len - 128..file_len].copy_from_slice(&tag);
        } else {
            // Append new tag
            file_data.extend_from_slice(&tag);
        }

        // Write modified file
        std::fs::write(&self.path, file_data)?;

        Ok(())
    }

    /// Write all metadata to FLAC file
    fn write_flac_metadata(&self, metadata: Metadata) -> PyResult<()> {
        // Read the whole file
        let mut file_data = std::fs::read(&self.path)?;

        // Find Vorbis Comment block
        let mut pos = 4; // Skip FLAC signature
        let mut found_vorbis = false;

        while pos < file_data.len() {
            if pos + 4 > file_data.len() {
                break;
            }

            // Read block header
            let is_last = (file_data[pos] & 0x80) != 0;
            let block_type = file_data[pos] & 0x7F;

            if block_type == 4 { // Vorbis Comment block type
                // Read block length
                let block_length = (((file_data[pos + 1] as u32) << 16) |
                                  ((file_data[pos + 2] as u32) << 8) |
                                  (file_data[pos + 3] as u32)) as usize;

                let header_size = 4;
                let total_size = header_size + block_length;

                // Read existing Vorbis comment
                let vorbis_data = &file_data[pos + header_size..pos + total_size];
                if let Ok(mut vorbis) = flac::VorbisComment::read(&mut std::io::Cursor::new(vorbis_data)) {
                    // Update all fields
                    if let Some(title) = &metadata.title {
                        vorbis.set(flac::VorbisFields::TITLE, title);
                    }
                    if let Some(artist) = &metadata.artist {
                        vorbis.set(flac::VorbisFields::ARTIST, artist);
                    }
                    if let Some(album) = &metadata.album {
                        vorbis.set(flac::VorbisFields::ALBUM, album);
                    }
                    if let Some(year) = &metadata.year {
                        vorbis.set(flac::VorbisFields::DATE, year);
                    }
                    if let Some(track) = &metadata.track {
                        vorbis.set(flac::VorbisFields::TRACKNUMBER, track);
                    }
                    if let Some(genre) = &metadata.genre {
                        vorbis.set(flac::VorbisFields::GENRE, genre);
                    }
                    if let Some(comment) = &metadata.comment {
                        vorbis.set(flac::VorbisFields::COMMENT, comment);
                    }
                    if let Some(lyrics) = &metadata.lyrics {
                        vorbis.set(flac::VorbisFields::LYRICS, lyrics);
                    } else {
                        // Remove lyrics if None
                        vorbis.remove(flac::VorbisFields::LYRICS);
                    }

                    let new_vorbis_data = vorbis.to_bytes();

                    // Update block
                    let new_block_length = new_vorbis_data.len();
                    let mut new_header = [0u8; 4];
                    new_header[0] = if is_last { 0x80 | 4 } else { 4 };
                    new_header[1] = ((new_block_length >> 16) & 0xFF) as u8;
                    new_header[2] = ((new_block_length >> 8) & 0xFF) as u8;
                    new_header[3] = (new_block_length & 0xFF) as u8;

                    // Replace the block
                    let mut new_file_data = Vec::new();
                    new_file_data.extend_from_slice(&file_data[..pos]);
                    new_file_data.extend_from_slice(&new_header);
                    new_file_data.extend_from_slice(&new_vorbis_data);
                    new_file_data.extend_from_slice(&file_data[pos + total_size..]);

                    file_data = new_file_data;
                    found_vorbis = true;
                    break;
                }
            } else {
                // Move to next block
                let block_length: usize = (((file_data[pos + 1] as u32) << 16) |
                                          ((file_data[pos + 2] as u32) << 8) |
                                          (file_data[pos + 3] as u32)) as usize;
                pos += 4 + block_length;

                if is_last {
                    break;
                }
            }
        }

        if !found_vorbis {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "No Vorbis Comment block found in FLAC file"
            ));
        }

        // Write modified file
        std::fs::write(&self.path, file_data)?;

        Ok(())
    }
}

/// Public Python methods
#[pymethods]
impl AudioFile {
    /// Create a new AudioFile instance
    #[new]
    fn new(path: String) -> PyResult<Self> {
        let file_type = Self::detect_file_type(&path)?;
        Ok(AudioFile { path, file_type })
    }

    /// Get all metadata as JSON string
    fn get_metadata_json(&self) -> PyResult<String> {
        let metadata = self.read_metadata()?;
        serde_json::to_string(&metadata)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// Set metadata from JSON string
    fn set_metadata_from_json(&self, json_str: String) -> PyResult<()> {
        // Read existing metadata first to preserve file_type and version
        let mut metadata = self.read_metadata()?;

        // Parse JSON and update fields
        let updates: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid JSON: {}", e)))?;

        // Update each field if present in JSON
        if let Some(title) = updates.get("title").and_then(|v| v.as_str()) {
            metadata.title = if title.is_empty() { None } else { Some(title.to_string()) };
        }
        if let Some(artist) = updates.get("artist").and_then(|v| v.as_str()) {
            metadata.artist = if artist.is_empty() { None } else { Some(artist.to_string()) };
        }
        if let Some(album) = updates.get("album").and_then(|v| v.as_str()) {
            metadata.album = if album.is_empty() { None } else { Some(album.to_string()) };
        }
        if let Some(year) = updates.get("year").and_then(|v| v.as_str()) {
            metadata.year = if year.is_empty() { None } else { Some(year.to_string()) };
        }
        if let Some(track) = updates.get("track").and_then(|v| v.as_str()) {
            metadata.track = if track.is_empty() { None } else { Some(track.to_string()) };
        }
        if let Some(genre) = updates.get("genre").and_then(|v| v.as_str()) {
            metadata.genre = if genre.is_empty() { None } else { Some(genre.to_string()) };
        }
        if let Some(comment) = updates.get("comment").and_then(|v| v.as_str()) {
            metadata.comment = if comment.is_empty() { None } else { Some(comment.to_string()) };
        }
        if let Some(lyrics) = updates.get("lyrics").and_then(|v| v.as_str()) {
            metadata.lyrics = if lyrics.is_empty() { None } else { Some(lyrics.to_string()) };
        } else if updates.get("lyrics").is_some() {
            // Explicitly set to None if present but null
            metadata.lyrics = None;
        }

        // Update based on file type
        match self.file_type.as_str() {
            "id3v2" => self.write_id3v2_metadata(metadata),
            "id3v1" => self.write_id3v1_metadata(metadata),
            "flac" => self.write_flac_metadata(metadata),
            _ => Err(pyo3::exceptions::PyValueError::new_err(
                format!("Unsupported file type: {}", self.file_type)
            )),
        }
    }
}

/// Convert regular integer to synchsafe integer (7 bits per byte)
fn to_synchsafe(size: usize) -> u32 {
    let size = size as u32;
    // Synchsafe: 每个字节只使用 7 位
    // 我们需要将 32 位的值转换成 synchsafe 格式
    // 正确的转换方式是：将原始值的位分成 7 位一组，然后重新组合
    
    // 计算每个 7 位字节的值
    let b0 = (size >> 21) & 0x7F;  // bits 21-27
    let b1 = (size >> 14) & 0x7F;  // bits 14-20
    let b2 = (size >> 7) & 0x7F;   // bits 7-13
    let b3 = size & 0x7F;          // bits 0-6
    
    // 将这些字节组合成一个 32 位值
    // 注意：synchsafe integer 的字节表示就是 b0, b1, b2, b3
    // 所以我们只需要将它们放在正确的位置
    (b0 << 21) | (b1 << 14) | (b2 << 7) | b3
}

/// Create ID3v2 frame
fn create_id3v2_frame(frame_id: &str, frame_data: &[u8], version_major: u8) -> Vec<u8> {
    let mut frame = Vec::new();

    // Frame ID
    frame.extend_from_slice(frame_id.as_bytes());

    // Frame size
    let frame_size = frame_data.len();
    if version_major >= 4 {
        // ID3v2.4 uses synchsafe integers
        frame.push(((frame_size >> 21) & 0x7F) as u8);
        frame.push(((frame_size >> 14) & 0x7F) as u8);
        frame.push(((frame_size >> 7) & 0x7F) as u8);
        frame.push((frame_size & 0x7F) as u8);
    } else {
        // ID3v2.3 uses regular integers
        frame.push(((frame_size >> 24) & 0xFF) as u8);
        frame.push(((frame_size >> 16) & 0xFF) as u8);
        frame.push(((frame_size >> 8) & 0xFF) as u8);
        frame.push((frame_size & 0xFF) as u8);
    }

    // Frame flags (all zero)
    frame.push(0);
    frame.push(0);

    // Frame data
    frame.extend_from_slice(frame_data);

    frame
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
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Metadata {
    #[pyo3(get)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub file_type: String,
    #[pyo3(get)]
    #[serde(skip_serializing_if = "String::is_empty")]
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
    #[pyo3(get, set)]
    pub lyrics: Option<String>,
}

#[pymethods]
impl Metadata {
    /// Create a new Metadata instance
    #[new]
    fn new() -> Self {
        Metadata::default()
    }

    /// Convert to dictionary
    fn to_dict<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, pyo3::types::PyDict>> {
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
        dict.set_item("lyrics", self.lyrics.as_ref())?;
        Ok(dict)
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
