// I/O utilities for reading audio files

use std::io::{Read, Seek, SeekFrom};

/// Read big-endian 16-bit integer
#[allow(dead_code)]
pub fn read_be_u16<R: Read>(reader: &mut R) -> std::io::Result<u16> {
    let mut buffer = [0u8; 2];
    reader.read_exact(&mut buffer)?;
    Ok(u16::from_be_bytes(buffer))
}

/// Read big-endian 32-bit integer
#[allow(dead_code)]
pub fn read_be_u32<R: Read>(reader: &mut R) -> std::io::Result<u32> {
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer)?;
    Ok(u32::from_be_bytes(buffer))
}

/// Read little-endian 16-bit integer
#[allow(dead_code)]
pub fn read_le_u16<R: Read>(reader: &mut R) -> std::io::Result<u16> {
    let mut buffer = [0u8; 2];
    reader.read_exact(&mut buffer)?;
    Ok(u16::from_le_bytes(buffer))
}

/// Read little-endian 32-bit integer
#[allow(dead_code)]
pub fn read_le_u32<R: Read>(reader: &mut R) -> std::io::Result<u32> {
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

/// Read synchsafe 32-bit integer (7 bits per byte)
#[allow(dead_code)]
pub fn read_synchsafe_u32<R: Read>(reader: &mut R) -> std::io::Result<u32> {
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer)?;
    Ok(((buffer[0] as u32) << 21) |
       ((buffer[1] as u32) << 14) |
       ((buffer[2] as u32) << 7) |
       (buffer[3] as u32))
}

/// Check if file has signature at current position
#[allow(dead_code)]
pub fn check_signature<R: Read + Seek>(reader: &mut R, signature: &[u8]) -> std::io::Result<bool> {
    let pos = reader.stream_position()?;
    let mut buffer = vec![0u8; signature.len()];
    reader.read_exact(&mut buffer)?;
    reader.seek(SeekFrom::Start(pos))?;
    Ok(&buffer == signature)
}