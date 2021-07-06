use crate::shape::RectangleShape;
use crate::sprite::Sprite;
use crate::texture::TextureSource;
use crate::{Color, Graphics};
use std::collections::HashSet;
use tuber_common::transform::Transform2D;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::query::accessors::R;
use tuber_ecs::system::SystemBundle;

pub struct Image {
    pub width: f32,
    pub height: f32,
    pub texture: TextureSource,
}

pub struct Frame {
    pub width: f32,
    pub height: f32,
    pub color: Color,
}

pub struct Text {
    text: String,
    font: String,
}

impl Text {
    pub fn new(text: &str, font: &str) -> Self {
        Self {
            text: text.into(),
            font: font.into(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn font(&self) -> &str {
        &self.font
    }
    pub fn set_font(&mut self, font: &str) {
        self.font = font.to_string();
    }
}

pub struct NoViewTransform;
