use tuber::ecs::prelude::*;
use tuber::graphics::RectangleShape;
use tuber::{Engine, Position, State};

#[derive(Debug)]
struct Velocity(f32, f32);

struct GameState;
impl State for GameState {
    fn initialize(&mut self, world: &mut World) -> Schedule {
        world.insert(
            (),
            (0..25).map(|_| {
                let width = 10.0 + rand::random::<f32>() * 100.0;
                let height = 10.0 + rand::random::<f32>() * 100.0;
                (
                    Position {
                        x: rand::random::<f32>() * (800.0 - width),
                        y: rand::random::<f32>() * (600.0 - height),
                    },
                    Velocity(rand::random::<f32>() * 5.0, rand::random::<f32>() * 5.0),
                    RectangleShape {
                        width,
                        height,
                        color: (rand::random(), rand::random(), rand::random()),
                    },
                )
            }),
        );

        Schedule::builder()
            .add_system(
                SystemBuilder::new("wall_collision")
                    .with_query(<(Read<Position>, Read<RectangleShape>, Write<Velocity>)>::query())
                    .build(|_, world, _, query| {
                        for (position, shape, mut velocity) in query.iter_mut(world) {
                            if (position.x + shape.width + velocity.0 > 800.0)
                                || (position.x + velocity.0 < 0.0)
                            {
                                velocity.0 = -velocity.0;
                            }
                            if (position.y + shape.height + velocity.1 > 600.0)
                                || (position.y + velocity.1 < 0.0)
                            {
                                velocity.1 = -velocity.1;
                            }
                        }
                    }),
            )
            .add_system(
                SystemBuilder::new("move_system")
                    .with_query(<(Write<Position>, Read<Velocity>)>::query())
                    .build(|_, world, _, query| {
                        for (mut position, velocity) in query.iter_mut(world) {
                            position.x += velocity.0;
                            position.y += velocity.1;
                        }
                    }),
            )
            .build()
    }
}

fn main() {
    futures::executor::block_on(Engine::new("fjorgyn").ignite(Box::new(GameState))).unwrap();
}
