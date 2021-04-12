use std::fmt;
use std::fmt::{Display, Formatter};
use tuber::ecs::ecs::Ecs;
use tuber::ecs::query::accessors::{R, W};
use tuber::ecs::system::SystemBundle;
use tuber::graphics::{Graphics, GraphicsAPI, SquareShape};
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::*;

struct Position {
    x: f32,
    y: f32,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "x: {}, y: {}", self.x, self.y)?;
        Ok(())
    }
}

struct Velocity {
    x: f32,
    y: f32,
}

fn move_system(ecs: &mut Ecs) {
    for (mut position, mut velocity) in ecs.query::<(W<Position>, W<Velocity>)>() {
        if (position.x >= 600.0) || (position.x <= 0.0) {
            velocity.x = -velocity.x;
        }

        if (position.y >= 600.0) || (position.y <= 0.0) {
            velocity.y = -velocity.y;
        }

        position.x += velocity.x;
        position.y += velocity.y;
    }
}

fn log_position_system(ecs: &mut Ecs) {
    for (position,) in ecs.query::<(R<Position>,)>() {
        println!("{}", position);
    }
}

fn main() -> tuber::Result<()> {
    let mut engine = Engine::new();
    engine.ecs().insert((
        Position { x: 5.0, y: 5.0 },
        Velocity { x: 1.0, y: 0.5 },
        SquareShape,
    ));

    let mut runner = WinitTuberRunner;
    let mut graphics = Graphics::new(Box::new(GraphicsWGPU::new()));

    let mut bundle = SystemBundle::new();
    bundle.add_system(move_system);
    bundle.add_system(log_position_system);
    engine.add_system_bundle(graphics.default_system_bundle());
    engine.add_system_bundle(bundle);

    runner.run(engine, graphics)
}
