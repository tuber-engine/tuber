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

pub enum TextureSource {
    WholeTexture(String),
    TextureRegion(String, TextureRegion),
}

impl TextureSource {
    pub(crate) fn texture_identifier(&self) -> String {
        match self {
            TextureSource::WholeTexture(texture_identifier) => texture_identifier,
            TextureSource::TextureRegion(texture_identifier, _) => texture_identifier,
        }
        .into()
    }

    pub(crate) fn normalized_texture_region(
        &self,
        texture_width: u32,
        texture_height: u32,
    ) -> TextureRegion {
        match self {
            TextureSource::WholeTexture(_) => TextureRegion::new(0.0, 0.0, 1.0, 1.0),
            TextureSource::TextureRegion(_, region) => TextureRegion {
                x: region.x / texture_width as f32,
                y: region.y / texture_height as f32,
                width: region.width / texture_width as f32,
                height: region.height / texture_height as f32,
            },
        }
    }
}

impl<T> From<T> for TextureSource
where
    T: ToString,
{
    fn from(str: T) -> Self {
        TextureSource::WholeTexture(str.to_string())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TextureRegion {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl TextureRegion {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

impl From<TextureRegion> for cgmath::Vector4<f32> {
    fn from(region: TextureRegion) -> Self {
        cgmath::Vector4::new(region.x, region.y, region.width, region.height)
    }
}
