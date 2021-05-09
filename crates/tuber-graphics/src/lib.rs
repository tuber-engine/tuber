use crate::camera::{Active, OrthographicCamera};
use crate::texture::{TextureData, TextureRegion, TextureSource};
use cgmath::{vec3, Deg};
use image::ImageError;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::query::accessors::{R, W};
use tuber_ecs::system::SystemBundle;
use tuber_ecs::EntityIndex;

#[derive(Debug)]
pub enum GraphicsError {
    TextureFileOpenFailure(std::io::Error),
    AtlasDescriptionFileOpenError(std::io::Error),
    ImageDecodeError(ImageError),
    SerdeError(serde_json::error::Error),
}

pub mod camera;
pub mod texture;

pub type Color = (f32, f32, f32);

pub struct Sprite {
    pub width: f32,
    pub height: f32,
    pub texture: TextureSource,
}

pub struct AnimatedSprite {
    pub width: f32,
    pub height: f32,
    pub texture: TextureSource,
    pub keyframes: Vec<TextureRegion>,
    pub current_keyframe: usize,
}

pub struct RectangleShape {
    pub width: f32,
    pub height: f32,
    pub color: Color,
}

#[derive(Debug, Copy, Clone)]
pub struct Transform2D {
    pub translation: (f32, f32),
    pub angle: f32,
    pub rotation_center: (f32, f32),
}

impl From<Transform2D> for cgmath::Matrix4<f32> {
    fn from(transform_2d: Transform2D) -> Self {
        let translate_to_rotation_center = vec3(
            transform_2d.rotation_center.0,
            transform_2d.rotation_center.1,
            0.0,
        );

        cgmath::Matrix4::from_translation(vec3(
            transform_2d.translation.0,
            transform_2d.translation.1,
            0.0,
        )) * cgmath::Matrix4::from_translation(translate_to_rotation_center.clone())
            * cgmath::Matrix4::from_angle_z(Deg(transform_2d.angle))
            * cgmath::Matrix4::from_translation(-translate_to_rotation_center)
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            translation: (0.0, 0.0),
            angle: 0.0,
            rotation_center: (0.0, 0.0),
        }
    }
}

pub type WindowSize = (u32, u32);
pub struct Window<'a>(pub Box<&'a dyn HasRawWindowHandle>);
unsafe impl HasRawWindowHandle for Window<'_> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0.raw_window_handle()
    }
}

pub struct TextureMetadata {
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize)]
pub struct TextureAtlas {
    texture_identifier: String,
    textures: HashMap<String, TextureRegion>,
}

impl TextureAtlas {
    pub fn texture_region(&self, texture_name: &str) -> Option<TextureRegion> {
        self.textures.get(texture_name).cloned()
    }
}

pub struct Graphics {
    graphics_impl: Box<dyn LowLevelGraphicsAPI>,
    texture_metadata: HashMap<String, TextureMetadata>,
    texture_atlases: HashMap<String, TextureAtlas>,
    bounding_box_rendering: bool,
}

impl Graphics {
    pub fn new(graphics_impl: Box<dyn LowLevelGraphicsAPI>) -> Self {
        Self {
            graphics_impl,
            texture_metadata: HashMap::new(),
            texture_atlases: HashMap::new(),
            bounding_box_rendering: false,
        }
    }
    pub fn initialize(&mut self, window: Window, window_size: (u32, u32)) {
        self.graphics_impl.initialize(window, window_size);
    }

    fn render(&mut self) {
        self.graphics_impl.render();
    }

    fn prepare_rectangle(&mut self, rectangle: &RectangleShape, transform: &Transform2D) {
        self.graphics_impl.prepare_quad(
            &QuadDescription {
                width: rectangle.width,
                height: rectangle.height,
                color: rectangle.color,
                texture: None,
            },
            transform,
            self.bounding_box_rendering,
        );
    }

    fn load_texture_atlas(&mut self, texture_atlas_identifier: &str) -> Result<(), GraphicsError> {
        let atlas_description_file = File::open(texture_atlas_identifier)
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
            .insert(texture_atlas_identifier.to_owned(), texture_atlas);
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

        let current_keyframe = animated_sprite.keyframes[animated_sprite.current_keyframe];
        self.graphics_impl.prepare_quad(
            &QuadDescription {
                width: animated_sprite.width,
                height: animated_sprite.height,
                color: (1.0, 1.0, 1.0),
                texture: Some(TextureDescription {
                    identifier: texture,
                    texture_region: TextureRegion::new(
                        current_keyframe.x,
                        current_keyframe.y,
                        current_keyframe.width,
                        current_keyframe.height,
                    )
                    .normalize(texture_width, texture_height),
                }),
            },
            transform,
            self.bounding_box_rendering,
        );

        Ok(())
    }

    fn prepare_sprite(
        &mut self,
        sprite: &Sprite,
        transform: &Transform2D,
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
            self.bounding_box_rendering,
        );
        Ok(())
    }

    pub fn default_system_bundle(&mut self) -> SystemBundle {
        let mut system_bundle = SystemBundle::new();
        system_bundle.add_system(sprite_animation_step_system);
        system_bundle.add_system(render_system);
        system_bundle
    }

    pub fn set_bounding_box_rendering(&mut self, enabled: bool) {
        self.bounding_box_rendering = enabled;
    }
}

pub fn render_system(ecs: &mut Ecs) {
    let mut graphics = ecs.resource_mut::<Graphics>();

    let (camera_id, (camera, _, camera_transform)) = ecs
        .query_one::<(R<OrthographicCamera>, R<Active>, R<Transform2D>)>()
        .expect("There is no camera");
    graphics
        .graphics_impl
        .update_camera(camera_id, &camera, &camera_transform);

    for (_, (rectangle_shape, transform)) in ecs.query::<(R<RectangleShape>, R<Transform2D>)>() {
        graphics.prepare_rectangle(&rectangle_shape, &transform);
    }
    for (_, (sprite, transform)) in ecs.query::<(R<Sprite>, R<Transform2D>)>() {
        graphics.prepare_sprite(&sprite, &transform).unwrap();
    }
    for (_, (animated_sprite, transform)) in ecs.query::<(R<AnimatedSprite>, R<Transform2D>)>() {
        graphics
            .prepare_animated_sprite(&animated_sprite, &transform)
            .unwrap();
    }
    graphics.render();
}

pub fn sprite_animation_step_system(ecs: &mut Ecs) {
    for (_, (mut animated_sprite,)) in ecs.query::<(W<AnimatedSprite>,)>() {
        animated_sprite.current_keyframe =
            (animated_sprite.current_keyframe + 1) % animated_sprite.keyframes.len();
    }
}

pub trait LowLevelGraphicsAPI {
    fn initialize(&mut self, window: Window, window_size: WindowSize);
    fn render(&mut self);
    fn prepare_quad(
        &mut self,
        quad_description: &QuadDescription,
        transform: &Transform2D,
        bounding_box_rendering: bool,
    );
    fn is_texture_in_memory(&self, texture_identifier: &str) -> bool;
    fn load_texture(&mut self, texture_data: TextureData);
    fn update_camera(
        &mut self,
        camera_id: EntityIndex,
        camera: &OrthographicCamera,
        transform: &Transform2D,
    );
}

pub struct QuadDescription {
    pub width: f32,
    pub height: f32,
    pub color: Color,
    pub texture: Option<TextureDescription>,
}

pub struct TextureDescription {
    pub identifier: String,
    pub texture_region: TextureRegion,
}
