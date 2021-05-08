use crate::camera::{Active, OrthographicCamera};
use crate::texture::{TextureData, TextureRegion, TextureSource};
use cgmath::{vec3, Deg};
use image::ImageError;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::collections::HashMap;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::query::accessors::R;
use tuber_ecs::system::SystemBundle;
use tuber_ecs::EntityIndex;

#[derive(Debug)]
pub enum GraphicsError {
    TextureFileOpenFailure(std::io::Error),
    ImageDecodeError(ImageError),
}

pub mod camera;
pub mod texture;

pub type Color = (f32, f32, f32);

pub struct Sprite {
    pub width: f32,
    pub height: f32,
    pub texture: TextureSource,
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

pub struct Graphics {
    graphics_impl: Box<dyn LowLevelGraphicsAPI>,
    texture_metadata: HashMap<String, TextureMetadata>,
    bounding_box_rendering: bool,
}

impl Graphics {
    pub fn new(graphics_impl: Box<dyn LowLevelGraphicsAPI>) -> Self {
        Self {
            graphics_impl,
            texture_metadata: HashMap::new(),
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

    fn prepare_sprite(
        &mut self,
        sprite: &Sprite,
        transform: &Transform2D,
    ) -> Result<(), GraphicsError> {
        let texture = sprite.texture.texture_identifier();
        if !self.graphics_impl.is_texture_in_memory(&texture) {
            if let Ok(texture_data) = TextureData::from_file(&texture) {
                self.texture_metadata.insert(
                    texture.clone(),
                    TextureMetadata {
                        width: texture_data.size.0,
                        height: texture_data.size.1,
                    },
                );
                self.graphics_impl.load_texture(texture_data);
            }
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
                    identifier: sprite.texture.texture_identifier(),
                    texture_region: sprite
                        .texture
                        .normalized_texture_region(texture_width, texture_height),
                }),
            },
            transform,
            self.bounding_box_rendering,
        );
        Ok(())
    }

    pub fn default_system_bundle(&mut self) -> SystemBundle {
        let mut system_bundle = SystemBundle::new();
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
        if let Err(e) = graphics.prepare_sprite(&sprite, &transform) {
            println!("{:?}", e);
        }
    }
    graphics.render();
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
