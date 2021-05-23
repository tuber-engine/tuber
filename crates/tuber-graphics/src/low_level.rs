use crate::*;

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
    fn on_window_resized(&mut self, size: WindowSize);
}

pub struct TextureDescription {
    pub identifier: String,
    pub texture_region: TextureRegion,
}

pub struct QuadDescription {
    pub width: f32,
    pub height: f32,
    pub color: Color,
    pub texture: Option<TextureDescription>,
}
