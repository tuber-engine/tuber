use crate::ecs::prelude::*;
use async_trait::async_trait;

pub type Color = (f32, f32, f32);

#[derive(Debug)]
pub struct RectangleShape {
    pub width: f32,
    pub height: f32,
    pub color: Color,
}

pub trait SceneRenderer {
    fn render(&mut self, world: &mut World);
}
