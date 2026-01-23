// FLAC PICTURE block implementation

use std::io::Read;

/// Picture types according to FLAC specification
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]  // Reserved for future use
pub enum PictureType {
    Other = 0,
    FileIcon = 1,
    OtherFileIcon = 2,
    CoverFront = 3,
    CoverBack = 4,
    LeafletPage = 5,
    Media = 6,
    LeadArtist = 7,
    Artist = 8,
    Conductor = 9,
    Band = 10,
    Composer = 11,
    Lyricist = 12,
    RecordingLocation = 13,
    DuringRecording = 14,
    DuringPerformance = 15,
    VideoScreenCapture = 16,
    BrightColouredFish = 17,
    Illustration = 18,
    BandLogo = 19,
    PublisherLogo = 20,
}

impl PictureType {
    #[allow(dead_code)]
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => PictureType::Other,
            1 => PictureType::FileIcon,
            2 => PictureType::OtherFileIcon,
            3 => PictureType::CoverFront,
            4 => PictureType::CoverBack,
            5 => PictureType::LeafletPage,
            6 => PictureType::Media,
            7 => PictureType::LeadArtist,
            8 => PictureType::Artist,
            9 => PictureType::Conductor,
            10 => PictureType::Band,
            11 => PictureType::Composer,
            12 => PictureType::Lyricist,
            13 => PictureType::RecordingLocation,
            14 => PictureType::DuringRecording,
            15 => PictureType::DuringPerformance,
            16 => PictureType::VideoScreenCapture,
            17 => PictureType::BrightColouredFish,
            18 => PictureType::Illustration,
            19 => PictureType::BandLogo,
            20 => PictureType::PublisherLogo,
            _ => PictureType::Other,
        }
    }

    #[allow(dead_code)]
pub fn to_string(&self) -> &'static str {
        match self {
            PictureType::Other => "Other",
            PictureType::FileIcon => "File Icon",
            PictureType::OtherFileIcon => "Other File Icon",
            PictureType::CoverFront => "Cover (front)",
            PictureType::CoverBack => "Cover (back)",
            PictureType::LeafletPage => "Leaflet page",
            PictureType::Media => "Media",
            PictureType::LeadArtist => "Lead artist",
            PictureType::Artist => "Artist",
            PictureType::Conductor => "Conductor",
            PictureType::Band => "Band",
            PictureType::Composer => "Composer",
            PictureType::Lyricist => "Lyricist",
            PictureType::RecordingLocation => "Recording Location",
            PictureType::DuringRecording => "During recording",
            PictureType::DuringPerformance => "During performance",
            PictureType::VideoScreenCapture => "Video screen capture",
            PictureType::BrightColouredFish => "Bright coloured fish",
            PictureType::Illustration => "Illustration",
            PictureType::BandLogo => "Band logo",
            PictureType::PublisherLogo => "Publisher logo",
        }
    }
}

/// FLAC PICTURE block structure
#[derive(Debug)]
#[allow(dead_code)]  // Reserved for future use
pub struct FlacPicture {
    #[allow(dead_code)]
    pub picture_type: PictureType,
    #[allow(dead_code)]
    pub mime_type: String,
    #[allow(dead_code)]
    pub description: String,
    #[allow(dead_code)]
    pub width: u32,
    #[allow(dead_code)]
    pub height: u32,
    #[allow(dead_code)]
    pub depth: u32,
    #[allow(dead_code)]
    pub colors: u32,
    #[allow(dead_code)]
    pub data: Vec<u8>,
}

impl FlacPicture {
    /// Read FLAC PICTURE block from data
    #[allow(dead_code)]
    pub fn read_from_data(data: &[u8]) -> std::io::Result<Self> {
        let mut cursor = std::io::Cursor::new(data);

        // Read picture type (32-bit big-endian)
        let mut type_bytes = [0u8; 4];
        cursor.read_exact(&mut type_bytes)?;
        let picture_type = PictureType::from_u32(u32::from_be_bytes(type_bytes));

        // Read MIME type length (32-bit big-endian)
        let mut mime_length_bytes = [0u8; 4];
        cursor.read_exact(&mut mime_length_bytes)?;
        let mime_length = u32::from_be_bytes(mime_length_bytes) as usize;

        // Read MIME type
        let mut mime_bytes = vec![0u8; mime_length];
        cursor.read_exact(&mut mime_bytes)?;
        let mime_type = String::from_utf8_lossy(&mime_bytes).to_string();

        // Read description length (32-bit big-endian)
        let mut desc_length_bytes = [0u8; 4];
        cursor.read_exact(&mut desc_length_bytes)?;
        let desc_length = u32::from_be_bytes(desc_length_bytes) as usize;

        // Read description (UTF-8)
        let mut desc_bytes = vec![0u8; desc_length];
        cursor.read_exact(&mut desc_bytes)?;
        let description = String::from_utf8_lossy(&desc_bytes).to_string();

        // Read width (32-bit big-endian)
        let mut width_bytes = [0u8; 4];
        cursor.read_exact(&mut width_bytes)?;
        let width = u32::from_be_bytes(width_bytes);

        // Read height (32-bit big-endian)
        let mut height_bytes = [0u8; 4];
        cursor.read_exact(&mut height_bytes)?;
        let height = u32::from_be_bytes(height_bytes);

        // Read depth (32-bit big-endian)
        let mut depth_bytes = [0u8; 4];
        cursor.read_exact(&mut depth_bytes)?;
        let depth = u32::from_be_bytes(depth_bytes);

        // Read colors (32-bit big-endian)
        let mut colors_bytes = [0u8; 4];
        cursor.read_exact(&mut colors_bytes)?;
        let colors = u32::from_be_bytes(colors_bytes);

        // Read picture data length (32-bit big-endian)
        let mut data_length_bytes = [0u8; 4];
        cursor.read_exact(&mut data_length_bytes)?;
        let data_length = u32::from_be_bytes(data_length_bytes) as usize;

        // Read picture data
        let mut picture_data = vec![0u8; data_length];
        cursor.read_exact(&mut picture_data)?;

        Ok(FlacPicture {
            picture_type,
            mime_type,
            description,
            width,
            height,
            depth,
            colors,
            data: picture_data,
        })
    }

    /// Get file extension based on MIME type
    #[allow(dead_code)]
    pub fn get_extension(&self) -> &'static str {
        match self.mime_type.as_str() {
            "image/jpeg" | "image/jpg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            "image/bmp" => "bmp",
            "image/tiff" => "tiff",
            _ => "jpg",
        }
    }

    /// Encode FlacPicture to bytes
    #[allow(dead_code)]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        // Picture type (32-bit big-endian)
        result.extend_from_slice(&(self.picture_type as u32).to_be_bytes());

        // MIME type length (32-bit big-endian)
        result.extend_from_slice(&(self.mime_type.len() as u32).to_be_bytes());
        // MIME type
        result.extend_from_slice(self.mime_type.as_bytes());

        // Description length (32-bit big-endian)
        result.extend_from_slice(&(self.description.len() as u32).to_be_bytes());
        // Description (UTF-8)
        result.extend_from_slice(self.description.as_bytes());

        // Width (32-bit big-endian)
        result.extend_from_slice(&self.width.to_be_bytes());

        // Height (32-bit big-endian)
        result.extend_from_slice(&self.height.to_be_bytes());

        // Depth (32-bit big-endian)
        result.extend_from_slice(&self.depth.to_be_bytes());

        // Colors (32-bit big-endian)
        result.extend_from_slice(&self.colors.to_be_bytes());

        // Picture data length (32-bit big-endian)
        result.extend_from_slice(&(self.data.len() as u32).to_be_bytes());
        // Picture data
        result.extend_from_slice(&self.data);

        result
    }

    /// Create a new FlacPicture from image data
    #[allow(dead_code)]
    pub fn new(data: Vec<u8>, mime_type: String, description: String) -> Self {
        FlacPicture {
            picture_type: PictureType::CoverFront,
            mime_type,
            description,
            width: 0,
            height: 0,
            depth: 0,
            colors: 0,
            data,
        }
    }
}