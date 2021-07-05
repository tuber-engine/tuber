use crate::GraphicsError;
use crate::GraphicsError::{ImageDecodeError, TextureFileOpenError};
use nalgebra::Vector4;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type TextureSize = (u32, u32);

pub struct TextureData {
    pub identifier: String,
    pub size: TextureSize,
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
            .map_err(|e| TextureFileOpenError(e))?
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

#[derive(Clone)]
pub enum TextureSource {
    WholeTexture(String),
    TextureRegion(String, TextureRegion),
    TextureAtlas(String, String),
}

impl TextureSource {
    pub(crate) fn texture_identifier(
        &self,
        texture_atlases: &HashMap<String, TextureAtlas>,
    ) -> String {
        match self {
            TextureSource::WholeTexture(texture_identifier) => texture_identifier,
            TextureSource::TextureRegion(texture_identifier, _) => texture_identifier,
            TextureSource::TextureAtlas(texture_atlas_identifier, _) => {
                &texture_atlases
                    .get(texture_atlas_identifier)
                    .expect("Texture atlas not found")
                    .texture_identifier
            }
        }
        .into()
    }

    pub(crate) fn normalized_texture_region(
        &self,
        texture_width: u32,
        texture_height: u32,
        texture_atlases: &HashMap<String, TextureAtlas>,
    ) -> TextureRegion {
        match self {
            TextureSource::WholeTexture(_) => TextureRegion::new(0.0, 0.0, 1.0, 1.0),
            TextureSource::TextureRegion(_, region) => TextureRegion {
                x: region.x / texture_width as f32,
                y: region.y / texture_height as f32,
                width: region.width / texture_width as f32,
                height: region.height / texture_height as f32,
            },
            TextureSource::TextureAtlas(texture_atlas, texture_name) => {
                let region = texture_atlases
                    .get(texture_atlas)
                    .expect("Texture atlas not found")
                    .textures
                    .get(texture_name)
                    .expect("Texture not found in atlas");

                TextureRegion {
                    x: region.x / texture_width as f32,
                    y: region.y / texture_height as f32,
                    width: region.width / texture_width as f32,
                    height: region.height / texture_height as f32,
                }
            }
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

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct TextureRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
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

    pub fn normalize(self, texture_width: u32, texture_height: u32) -> Self {
        let texture_width = texture_width as f32;
        let texture_height = texture_height as f32;
        Self {
            x: self.x / texture_width,
            y: self.y / texture_height,
            width: self.width / texture_width,
            height: self.height / texture_height,
        }
    }
}

impl From<TextureRegion> for Vector4<f32> {
    fn from(region: TextureRegion) -> Self {
        Vector4::new(region.x, region.y, region.width, region.height)
    }
}

pub struct TextureMetadata {
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize)]
pub struct TextureAtlas {
    pub texture_identifier: String,
    pub textures: HashMap<String, TextureRegion>,
}

impl TextureAtlas {
    pub fn texture_region(&self, texture_name: &str) -> Option<TextureRegion> {
        self.textures.get(texture_name).cloned()
    }

    pub fn texture_identifier(&self) -> &str {
        &self.texture_identifier
    }
}
