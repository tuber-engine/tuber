use async_trait::async_trait;
use tecs::core::Ecs;

pub type Color = (f32, f32, f32);

#[derive(Debug)]
pub struct SquareShape {
    pub width: f32,
    pub height: f32,
    pub color: Color,
}

pub trait SceneRenderer {
    fn render(&mut self, ecs: &mut Ecs);
}
