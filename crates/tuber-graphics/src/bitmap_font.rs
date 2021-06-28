use crate::GraphicsError;
use std::str::FromStr;
    use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::texture::TextureRegion;

#[derive(Serialize, Deserialize)]
pub struct BitmapFont {
    /// Identifier of the bitmap font
    identifier: String,
    /// Identifier of the texture atlas
    font_atlas_identifier: String,
    /// The region of the bitmap font on the texture atlas
    font_atlas_region: TextureRegion,
    /// The glyphs data
    glyphs: HashMap<char, BitmapGlyph>
}


impl BitmapFont {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn texture_atlas_identifier(&self) -> &str {
        &self.font_atlas_identifier
    }

    pub fn texture_atlas_region(&self) -> TextureRegion {
        self.font_atlas_region
    }

    pub fn glyph(&self, character: char) -> Option<&BitmapGlyph> {
        self.glyphs.get(&character)
    }

    pub fn from_file(path: &str) -> Result<Self, GraphicsError> {
       Self::from_str(&std::fs::read_to_string(path).map_err(|error| GraphicsError::BitmapFontFileReadError(error))?)
    }
}

impl FromStr for BitmapFont {
    type Err = GraphicsError;

    fn from_str(json_string: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(json_string).map_err(|e| GraphicsError::SerdeError(e))
    }
}


#[derive(Serialize, Deserialize)]
pub struct BitmapGlyph {
    region: TextureRegion
}

impl BitmapGlyph {
    pub fn region(&self) -> &TextureRegion {
        &self.region
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_from_json() -> Result<(), GraphicsError> {
        let json = r#"
        {
            "identifier": "font",
            "font_atlas_identifier": "font_atlas",
            "font_atlas_region": {
                "x": 0,
                "y": 0,
                "width": 0,
                "height": 0
            },
            "glyphs": {
                "A": {
                    "region": {
                        "x": 0,
                        "y": 0,
                        "width": 32,
                        "height": 32
                    }
                },
                "D": {
                    "region": {
                        "x": 32,
                        "y": 0,
                        "width": 32,
                        "height": 32
                    }
                }
            }
        }
        "#;

        let bitmap_font = BitmapFont::from_str(json)?;
        assert_eq!(bitmap_font.identifier, "font");
        assert_eq!(bitmap_font.font_atlas_identifier, "font_atlas");
        assert_eq!(bitmap_font.glyphs.len(), 2);
        assert!(bitmap_font.glyphs.contains_key(&'A'));
        assert!(bitmap_font.glyphs.contains_key(&'D'));
        Ok(())
    }
}


