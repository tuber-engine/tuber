use async_trait::async_trait;
use tecs::core::Ecs;

pub struct SquareShape {
    width: f32,
    height: f32,
}

pub trait SceneRenderer {
    fn render(&mut self);
}
