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
        write!(f, "x: {}, y: {}", self.x, self.y)?;
        Ok(())
    }
}

struct Velocity {
    x: f32,
    y: f32,
}

fn move_system(ecs: &mut Ecs) {
    for (_id, (mut position, mut velocity)) in ecs.query::<(W<Position>, W<Velocity>)>() {
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
    for (id, (position,)) in ecs.query::<(R<Position>,)>() {
        println!("{}: {}", id, position);
    }
}

fn log_collision_system(ecs: &mut Ecs) {
    for (first_entity_id, (first_position,)) in ecs.query::<(R<Position>,)>() {
        for (second_entity_id, (second_position,)) in ecs.query::<(R<Position>,)>() {
            if first_entity_id == second_entity_id {
                continue;
            }

            if first_position.x < second_position.x + 10.0
                && first_position.x + 10.0 > second_position.x
                && first_position.y < second_position.y + 10.0
                && first_position.y + 10.0 > second_position.y
            {
                println!(
                    "collision between {} and {}",
                    first_entity_id, second_entity_id
                );
            }
        }
    }
}

fn main() -> tuber::Result<()> {
    let mut engine = Engine::new();
    engine.ecs().insert((
        Position { x: 1.0, y: 1.0 },
        Velocity { x: 1.0, y: 0.5 },
        SquareShape,
    ));
    engine.ecs().insert((
        Position { x: 100.0, y: 20.0 },
        Velocity { x: 3.0, y: 0.15 },
        SquareShape,
    ));
    engine.ecs().insert((
        Position { x: 30.0, y: 25.0 },
        Velocity { x: -1.5, y: -2.0 },
        SquareShape,
    ));
    engine.ecs().insert((
        Position { x: 125.0, y: 125.0 },
        Velocity { x: -6.0, y: 4.5 },
        SquareShape,
    ));

    let mut runner = WinitTuberRunner;
    let mut graphics = Graphics::new(Box::new(GraphicsWGPU::new()));

    let mut bundle = SystemBundle::new();
    bundle.add_system(move_system);
    bundle.add_system(log_position_system);
    bundle.add_system(log_collision_system);
    engine.add_system_bundle(graphics.default_system_bundle());
    engine.add_system_bundle(bundle);

    runner.run(engine, graphics)
}
