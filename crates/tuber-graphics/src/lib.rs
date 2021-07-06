use crate::bitmap_font::BitmapFont;
use crate::camera::{Active, OrthographicCamera};
use crate::low_level::*;
use crate::shape::RectangleShape;
use crate::sprite::{sprite_animation_step_system, AnimatedSprite, Sprite};
use crate::texture::{TextureAtlas, TextureData, TextureMetadata, TextureRegion, TextureSource};
use crate::tilemap::TilemapRender;
use crate::ui::{Frame, Image, NoViewTransform, Text};
use image::ImageError;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use tuber_common::tilemap::Tilemap;
use tuber_common::transform::Transform2D;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::query::accessors::{R, W};
use tuber_ecs::system::SystemBundle;
use tuber_ecs::EntityIndex;

#[derive(Debug)]
pub enum GraphicsError {
    TextureFileOpenError(std::io::Error),
    AtlasDescriptionFileOpenError(std::io::Error),
    ImageDecodeError(ImageError),
    SerdeError(serde_json::error::Error),
    BitmapFontFileReadError(std::io::Error),
}

pub mod bitmap_font;
pub mod camera;
pub mod low_level;
pub mod shape;
pub mod sprite;
pub mod texture;
pub mod tilemap;
pub mod ui;

pub type Color = (f32, f32, f32);

pub type WindowSize = (u32, u32);
pub struct Window<'a>(pub Box<&'a dyn HasRawWindowHandle>);
unsafe impl HasRawWindowHandle for Window<'_> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0.raw_window_handle()
    }
}

pub struct Graphics {
    graphics_impl: Box<dyn LowLevelGraphicsAPI>,
    texture_metadata: HashMap<String, TextureMetadata>,
    texture_atlases: HashMap<String, TextureAtlas>,
    fonts: HashMap<String, BitmapFont>,
    bounding_box_rendering: bool,
}

impl Graphics {
    pub fn new(graphics_impl: Box<dyn LowLevelGraphicsAPI>) -> Self {
        Self {
            graphics_impl,
            texture_metadata: HashMap::new(),
            texture_atlases: HashMap::new(),
            fonts: Default::default(),
            bounding_box_rendering: false,
        }
    }
    pub fn initialize(&mut self, window: Window, window_size: (u32, u32)) {
        self.graphics_impl.initialize(window, window_size);
    }

    fn render(&mut self) {
        self.graphics_impl.render();
    }

    pub fn prepare_rectangle(
        &mut self,
        rectangle: &RectangleShape,
        transform: &Transform2D,
        apply_view_transform: bool,
    ) {
        self.graphics_impl.prepare_quad(
            &QuadDescription {
                width: rectangle.width,
                height: rectangle.height,
                color: rectangle.color,
                texture: None,
            },
            transform,
            apply_view_transform,
            self.bounding_box_rendering,
        );
    }

    fn load_texture_atlas(&mut self, texture_atlas_path: &str) -> Result<(), GraphicsError> {
        let atlas_description_file = File::open(texture_atlas_path)
            .map_err(|e| GraphicsError::AtlasDescriptionFileOpenError(e))?;
        let reader = BufReader::new(atlas_description_file);
        let texture_atlas: TextureAtlas =
            serde_json::from_reader(reader).map_err(|e| GraphicsError::SerdeError(e))?;

        if !self
            .graphics_impl
            .is_texture_in_memory(&texture_atlas.texture_identifier)
        {
            self.load_texture(&texture_atlas.texture_identifier);
        }

        self.texture_atlases
            .insert(texture_atlas_path.to_owned(), texture_atlas);
        Ok(())
    }

    fn load_texture(&mut self, texture: &str) {
        if let Ok(texture_data) = TextureData::from_file(&texture) {
            self.texture_metadata.insert(
                texture.to_owned(),
                TextureMetadata {
                    width: texture_data.size.0,
                    height: texture_data.size.1,
                },
            );
            self.graphics_impl.load_texture(texture_data);
        }
    }

    fn prepare_animated_sprite(
        &mut self,
        animated_sprite: &AnimatedSprite,
        transform: &Transform2D,
        apply_view_transform: bool,
    ) -> Result<(), GraphicsError> {
        if let TextureSource::TextureAtlas(texture_atlas_identifier, _) = &animated_sprite.texture {
            if !self.texture_atlases.contains_key(texture_atlas_identifier) {
                self.load_texture_atlas(texture_atlas_identifier)?;
            }
        }

        let texture = animated_sprite
            .texture
            .texture_identifier(&self.texture_atlases);
        if !self.graphics_impl.is_texture_in_memory(&texture) {
            self.load_texture(&texture);
        }

        let (texture_width, texture_height) = match self.texture_metadata.get(&texture) {
            Some(metadata) => (metadata.width, metadata.height),
            None => (32, 32),
        };

        let current_keyframe = animated_sprite.animation_state.keyframes
            [animated_sprite.animation_state.current_keyframe];

        let mut normalized_texture_region = TextureRegion::new(
            current_keyframe.x,
            current_keyframe.y,
            current_keyframe.width,
            current_keyframe.height,
        )
        .normalize(texture_width, texture_height);

        if animated_sprite.animation_state.flip_x {
            normalized_texture_region = normalized_texture_region.flip_x();
        }

        self.graphics_impl.prepare_quad(
            &QuadDescription {
                width: animated_sprite.width,
                height: animated_sprite.height,
                color: (1.0, 1.0, 1.0),
                texture: Some(TextureDescription {
                    identifier: texture,
                    texture_region: normalized_texture_region,
                }),
            },
            transform,
            apply_view_transform,
            self.bounding_box_rendering,
        );

        Ok(())
    }

    pub fn prepare_sprite(
        &mut self,
        sprite: &Sprite,
        transform: &Transform2D,
        apply_view_transform: bool,
    ) -> Result<(), GraphicsError> {
        if let TextureSource::TextureAtlas(texture_atlas_identifier, _) = &sprite.texture {
            if !self.texture_atlases.contains_key(texture_atlas_identifier) {
                self.load_texture_atlas(texture_atlas_identifier)?;
            }
        }

        let texture = sprite.texture.texture_identifier(&self.texture_atlases);
        if !self.graphics_impl.is_texture_in_memory(&texture) {
            self.load_texture(&texture);
        }

        let (texture_width, texture_height) = match self.texture_metadata.get(&texture) {
            Some(metadata) => (metadata.width, metadata.height),
            None => (32, 32),
        };
        self.graphics_impl.prepare_quad(
            &QuadDescription {
                width: sprite.width,
                height: sprite.height,
                color: (1.0, 1.0, 1.0),
                texture: Some(TextureDescription {
                    identifier: texture,
                    texture_region: sprite.texture.normalized_texture_region(
                        texture_width,
                        texture_height,
                        &self.texture_atlases,
                    ),
                }),
            },
            transform,
            apply_view_transform,
            self.bounding_box_rendering,
        );
        Ok(())
    }

    fn prepare_tilemap(
        &mut self,
        tilemap: &Tilemap,
        tilemap_render: &TilemapRender,
        transform: &Transform2D,
    ) {
        if !self
            .texture_atlases
            .contains_key(&tilemap_render.texture_atlas_identifier)
        {
            self.load_texture_atlas(&tilemap_render.texture_atlas_identifier)
                .unwrap();
        }

        self.graphics_impl.prepare_tilemap(
            tilemap,
            tilemap_render,
            self.texture_atlases
                .get(&tilemap_render.texture_atlas_identifier)
                .unwrap(),
            transform,
        );
    }

    pub fn prepare_text(
        &mut self,
        text: &str,
        font_path: &str,
        transform: &Transform2D,
        apply_view_transform: bool,
    ) {
        if !self.fonts.contains_key(font_path) {
            self.load_font(font_path).expect("Font not found");
        }
        let font_atlas_path = self.fonts[font_path].font_atlas_path().to_owned();
        if !self.texture_atlases.contains_key(&font_atlas_path) {
            self.load_texture_atlas(&font_atlas_path).unwrap();
        }

        let font = &self.fonts[font_path];
        let texture_atlas = &self.texture_atlases[font.font_atlas_path()];

        let texture_identifier = texture_atlas.texture_identifier();
        let texture = &self.texture_metadata[texture_identifier];
        let font_region = texture_atlas
            .texture_region(font_path)
            .expect("Font region not found");

        let mut offset_x = transform.translation.0;
        let mut offset_y = transform.translation.1;
        for character in text.chars() {
            if character == '\n' {
                offset_y += (font.line_height() + font.line_spacing()) as f32;
                offset_x = transform.translation.0;
                continue;
            }

            let glyph_data = if font.ignore_case() {
                if let Some(glyph) = font.glyph(character.to_ascii_uppercase()) {
                    glyph
                } else {
                    font.glyph(character.to_ascii_lowercase())
                        .expect("Glyph not found")
                }
            } else {
                font.glyph(character).expect("Glyph not found")
            };

            let glyph_region = glyph_data.region();
            let mut glyph_transform = transform.clone();
            glyph_transform.translation.0 = offset_x;
            glyph_transform.translation.1 = offset_y;
            glyph_transform.rotation_center = (-offset_x, -offset_y);

            self.graphics_impl.prepare_quad(
                &QuadDescription {
                    width: glyph_region.width,
                    height: glyph_region.height,
                    color: (0.0, 0.0, 0.0),
                    texture: Some(TextureDescription {
                        identifier: texture_identifier.into(),
                        texture_region: TextureRegion {
                            x: (font_region.x + glyph_region.x) / texture.width as f32,
                            y: (font_region.y + glyph_region.y) / texture.height as f32,
                            width: glyph_region.width / texture.width as f32,
                            height: glyph_region.height / texture.height as f32,
                        },
                    }),
                },
                &glyph_transform,
                apply_view_transform,
                false,
            );

            offset_x += glyph_region.width + font.letter_spacing() as f32;
        }
    }

    fn load_font(&mut self, font_path: &str) -> Result<(), GraphicsError> {
        let font = BitmapFont::from_file(font_path)?;
        self.fonts.insert(font_path.into(), font);
        Ok(())
    }

    pub fn default_system_bundle() -> SystemBundle {
        let mut system_bundle = SystemBundle::new();
        system_bundle.add_system(sprite_animation_step_system);
        system_bundle
    }

    pub fn set_clear_color(&mut self, clear_color: Color) {
        self.graphics_impl.set_clear_color(clear_color);
    }

    pub fn set_bounding_box_rendering(&mut self, enabled: bool) {
        self.bounding_box_rendering = enabled;
    }

    pub fn on_window_resized(&mut self, width: u32, height: u32) {
        self.graphics_impl.on_window_resized((width, height));
    }
}

pub fn render(ecs: &mut Ecs) {
    let mut graphics = ecs.shared_resource_mut::<Graphics>().unwrap();

    let (camera_id, (camera, _, camera_transform)) = ecs
        .query_one::<(R<OrthographicCamera>, R<Active>, R<Transform2D>)>()
        .expect("There is no camera");
    graphics
        .graphics_impl
        .update_camera(camera_id, &camera, &camera_transform);

    for (_, (tilemap, tilemap_render, transform)) in
        ecs.query::<(R<Tilemap>, R<TilemapRender>, R<Transform2D>)>()
    {
        graphics.prepare_tilemap(&tilemap, &tilemap_render, &transform);
    }

    for (_, (rectangle_shape, transform)) in ecs.query::<(R<RectangleShape>, R<Transform2D>)>() {
        graphics.prepare_rectangle(&rectangle_shape, &transform, true);
    }
    for (_, (sprite, transform)) in ecs.query::<(R<Sprite>, R<Transform2D>)>() {
        graphics.prepare_sprite(&sprite, &transform, true).unwrap();
    }
    for (_, (animated_sprite, transform)) in ecs.query::<(R<AnimatedSprite>, R<Transform2D>)>() {
        graphics
            .prepare_animated_sprite(&animated_sprite, &transform, true)
            .unwrap();
    }

    for (_, (mut tilemap_render,)) in ecs.query::<(W<TilemapRender>,)>() {
        tilemap_render.dirty = false;
    }

    for (id, (frame, transform)) in ecs.query::<(R<Frame>, R<Transform2D>)>() {
        let apply_view_transform = !ecs.query_one_by_id::<(R<NoViewTransform>,)>(id).is_some();
        graphics.prepare_rectangle(
            &RectangleShape {
                width: frame.width,
                height: frame.height,
                color: frame.color,
            },
            &transform,
            apply_view_transform,
        );
    }

    for (id, (text, transform)) in ecs.query::<(R<Text>, R<Transform2D>)>() {
        let apply_view_transform = !ecs.query_one_by_id::<(R<NoViewTransform>,)>(id).is_some();
        graphics.prepare_text(text.text(), text.font(), &transform, apply_view_transform);
    }

    for (id, (image, transform)) in ecs.query::<(R<Image>, R<Transform2D>)>() {
        let apply_view_transform = !ecs.query_one_by_id::<(R<NoViewTransform>,)>(id).is_some();
        let sprite = Sprite {
            width: image.width,
            height: image.height,
            texture: image.texture.clone(),
        };

        graphics.prepare_sprite(&sprite, &transform, apply_view_transform);
    }

    graphics.render();
}
