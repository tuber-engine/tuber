use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use tuber_ecs::system::SystemBundle;

pub struct RectangleShape {
    pub width: f32,
    pub height: f32,
    pub color: (f32, f32, f32),
}

#[derive(Debug)]
pub struct Transform2D {
    pub translation: (f32, f32),
}

pub type WindowSize = (u32, u32);
pub struct Window<'a>(pub Box<&'a dyn HasRawWindowHandle>);
unsafe impl HasRawWindowHandle for Window<'_> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0.raw_window_handle()
    }
}

pub struct Graphics {
    graphics_impl: Box<dyn GraphicsAPI>,
}

impl Graphics {
    pub fn new(graphics_impl: Box<dyn GraphicsAPI>) -> Self {
        Self { graphics_impl }
    }
}

impl GraphicsAPI for Graphics {
    fn initialize(&mut self, window: Window, window_size: (u32, u32)) {
        self.graphics_impl.initialize(window, window_size);
    }

    fn default_system_bundle(&mut self) -> SystemBundle {
        self.graphics_impl.default_system_bundle()
    }

    fn render(&mut self) {
        self.graphics_impl.render();
    }

    fn prepare_rectangle(&mut self, rectangle: &RectangleShape, transform: &Transform2D) {
        self.graphics_impl.prepare_rectangle(rectangle, transform);
    }

    fn finish_prepare_render(&mut self) {
        self.graphics_impl.finish_prepare_render();
    }
}

pub trait GraphicsAPI {
    fn initialize(&mut self, window: Window, window_size: WindowSize);
    fn default_system_bundle(&mut self) -> SystemBundle;
    fn render(&mut self);
    fn prepare_rectangle(&mut self, rectangle: &RectangleShape, transform: &Transform2D);
    fn finish_prepare_render(&mut self);
}
