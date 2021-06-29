use crate::texture::TextureRegion;
use crate::GraphicsError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct BitmapFont {
    /// Path of the font atlas
    font_atlas_path: String,
    /// The region of the bitmap font on the texture atlas
    font_atlas_region: TextureRegion,
    /// The height of a line in pixels
    line_height: u32,
    /// The spacing between lines in pixels
    line_spacing: u32,
    /// The space between letters in pixels
    letter_spacing: u32,
    /// The flag specifying whether case must be taken in account when looking for a glyph
    ignore_case: bool,
    /// The glyphs data
    glyphs: HashMap<char, BitmapGlyph>,
}

impl BitmapFont {
    pub fn font_atlas_path(&self) -> &str {
        &self.font_atlas_path
    }

    pub fn font_atlas_region(&self) -> TextureRegion {
        self.font_atlas_region
    }

    pub fn glyph(&self, character: char) -> Option<&BitmapGlyph> {
        self.glyphs.get(&character)
    }

    pub fn line_height(&self) -> u32 {
        self.line_height
    }

    pub fn line_spacing(&self) -> u32 {
        self.line_spacing
    }

    pub fn letter_spacing(&self) -> u32 {
        self.letter_spacing
    }

    pub fn ignore_case(&self) -> bool {
        self.ignore_case
    }

    pub fn from_file(path: &str) -> Result<Self, GraphicsError> {
        Self::from_str(
            &std::fs::read_to_string(path)
                .map_err(|error| GraphicsError::BitmapFontFileReadError(error))?,
        )
    }
}

impl FromStr for BitmapFont {
    type Err = GraphicsError;

    fn from_str(json_string: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(json_string).map_err(|e| GraphicsError::SerdeError(e))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BitmapGlyph {
    region: TextureRegion,
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
            "font_atlas_path": "font_atlas",
            "font_atlas_region": {
                "x": 0,
                "y": 0,
                "width": 0,
                "height": 0
            },
            "letter_spacing": 2,
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
        assert_eq!(bitmap_font.font_atlas_path, "font_atlas");
        assert_eq!(bitmap_font.letter_spacing, 2);
        assert_eq!(bitmap_font.glyphs.len(), 2);
        assert!(bitmap_font.glyphs.contains_key(&'A'));
        assert!(bitmap_font.glyphs.contains_key(&'D'));
        Ok(())
    }
}
