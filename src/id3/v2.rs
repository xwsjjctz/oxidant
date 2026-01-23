// ID3v2 tag implementation

use std::io::Read;

/// ID3v2 header structure
#[derive(Debug)]
pub struct Id3v2Header {
    pub version: (u8, u8),
    #[allow(dead_code)]
    pub flags: u8,
    pub size: u32,
}

/// ID3v2 tag structure
#[derive(Debug)]
pub struct Id3v2Tag {
    #[allow(dead_code)]
    pub header: Id3v2Header,
    pub frames: Vec<Id3Frame>,
}

/// ID3v2 frame structure
#[derive(Debug)]
pub struct Id3Frame {
    pub frame_id: String,
    pub size: u32,
    #[allow(dead_code)]
    pub flags: u16,
    pub data: Vec<u8>,
}

impl Id3v2Header {
    const HEADER_SIZE: usize = 10;
    const ID: [u8; 3] = [b'I', b'D', b'3'];

    /// Read ID3v2 header from reader
    pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Option<Self>> {
        let mut buffer = [0u8; Self::HEADER_SIZE];
        reader.read_exact(&mut buffer)?;

        // Check for ID3 identifier
        if &buffer[0..3] != Self::ID {
            return Ok(None);
        }

        let version = (buffer[3], buffer[4]);
        let flags = buffer[5];
        let size = Self::parse_synchsafe(&buffer[6..10]);

        Ok(Some(Id3v2Header {
            version,
            flags,
            size,
        }))
    }

    /// Parse synchsafe integer (7 bits per byte)
    fn parse_synchsafe(bytes: &[u8]) -> u32 {
        ((bytes[0] as u32) << 21) |
        ((bytes[1] as u32) << 14) |
        ((bytes[2] as u32) << 7) |
        (bytes[3] as u32)
    }
}

impl Id3v2Tag {
    /// Read ID3v2 tag from reader
    pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Option<Self>> {
        let header = match Id3v2Header::read(reader)? {
            Some(h) => h,
            None => return Ok(None),
        };

        let mut frames = Vec::new();
        let mut remaining = header.size as usize;

        while remaining > 0 {
            let frame = match Id3Frame::read(reader, header.version)? {
                Some(f) => f,
                None => break,
            };

            let frame_total_size = frame.size as usize + 10; // frame header is 10 bytes
            if frame_total_size > remaining {
                break;
            }
            remaining -= frame_total_size;
            frames.push(frame);
        }

        Ok(Some(Id3v2Tag { header, frames }))
    }
}

impl Id3Frame {
    /// Read ID3v2 frame from reader
    pub fn read<R: Read>(reader: &mut R, version: (u8, u8)) -> std::io::Result<Option<Self>> {
        let mut buffer = [0u8; 10];
        reader.read_exact(&mut buffer)?;

        // Check for padding (all zeros)
        if buffer.iter().all(|&b| b == 0) {
            return Ok(None);
        }

        let frame_id = String::from_utf8_lossy(&buffer[0..4]).to_string();

        // Frame size parsing depends on version
        let size = if version.0 >= 4 {
            // ID3v2.4 uses synchsafe integers
            Id3v2Header::parse_synchsafe(&buffer[4..8])
        } else {
            // ID3v2.3 uses regular integers
            ((buffer[4] as u32) << 24) |
            ((buffer[5] as u32) << 16) |
            ((buffer[6] as u32) << 8) |
            (buffer[7] as u32)
        };

        let flags = ((buffer[8] as u16) << 8) | (buffer[9] as u16);

        // Read frame data
        let mut data = vec![0u8; size as usize];
        reader.read_exact(&mut data)?;

        Ok(Some(Id3Frame {
            frame_id,
            size,
            flags,
            data,
        }))
    }
}