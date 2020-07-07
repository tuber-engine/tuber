use tecs::{
    core::Ecs,
    query::{Imm, Mut},
    system::System,
};
use tuber::graphics::RectangleShape;
use tuber::{Engine, Position, State, SystemSchedule};

#[derive(Debug)]
struct Velocity(f32, f32);

struct GameState;
impl State for GameState {
    fn initialize(&mut self, ecs: &mut Ecs, systems: &mut SystemSchedule) {
        for _ in 0..25 {
            ecs.new_entity()
                .with_component(Position {
                    x: rand::random::<f32>() / 1.1f32,
                    y: rand::random::<f32>() / 1.1f32,
                })
                .with_component(Velocity(
                    rand::random::<f32>() / 50f32,
                    rand::random::<f32>() / 50f32,
                ))
                .with_component(RectangleShape {
                    width: 0.1 + rand::random::<f32>() / 10.0f32,
                    height: 0.1 + rand::random::<f32>() / 10.0f32,
                    color: (rand::random(), rand::random(), rand::random()),
                })
                .build();
        }

        systems.add_system(
            System::<(Mut<Position>, Imm<RectangleShape>, Mut<Velocity>)>::new(
                |(position, shape, velocity)| {
                    if (position.x + shape.width + velocity.0 > 1.0)
                        || (position.x + velocity.0 < 0.0)
                    {
                        velocity.0 = -velocity.0;
                    }
                    if (position.y + shape.height + velocity.1 > 1.0)
                        || (position.y + velocity.1 < 0.0)
                    {
                        velocity.1 = -velocity.1;
                    }

                    position.x += velocity.0;
                    position.y += velocity.1;
                },
            ),
        );
    }
}

fn main() {
    futures::executor::block_on(Engine::new("fjorgyn").ignite(Box::new(GameState))).unwrap();
}
