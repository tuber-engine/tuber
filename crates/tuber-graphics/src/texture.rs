use crate::GraphicsError;
use crate::GraphicsError::{ImageDecodeError, TextureFileOpenFailure};

pub struct TextureData {
    pub identifier: String,
    pub size: (u32, u32),
    pub bytes: Vec<u8>,
}

impl TextureData {
    pub fn from_bytes(identifier: &str, bytes: &[u8]) -> Result<TextureData, GraphicsError> {
        let image = image::load_from_memory(bytes).unwrap();
        let image = image.as_rgba8().unwrap();

        Ok(TextureData {
            identifier: identifier.into(),
            size: image.dimensions(),
            bytes: image.to_vec(),
        })
    }

    pub fn from_file(file_path: &str) -> Result<TextureData, GraphicsError> {
        use image::io::Reader as ImageReader;
        let image = ImageReader::open(file_path)
            .map_err(|e| TextureFileOpenFailure(e))?
            .decode()
            .map_err(|e| ImageDecodeError(e))?;
        let image = image.as_rgba8().unwrap();

        Ok(TextureData {
            identifier: file_path.into(),
            size: image.dimensions(),
            bytes: image.to_vec(),
        })
    }
}
