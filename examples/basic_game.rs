use tecs::{
    core::Ecs,
    query::{Imm, Mut},
    system::System,
};
use tuber::{Engine, State, SystemSchedule};

#[derive(Debug)]
struct Position(f32, f32);
#[derive(Debug)]
struct Velocity(f32, f32);

struct GameState;
impl State for GameState {
    fn initialize(&mut self, ecs: &mut Ecs, systems: &mut SystemSchedule) {
        ecs.new_entity()
            .with_component(Position(0f32, 0f32))
            .with_component(Velocity(0.001f32, 0.002f32))
            .build();

        // Print system
        systems.add_system(System::<(Imm<Position>,)>::new(|(position,)| {
            println!("{:?}", position);
        }));

        // Move system
        systems.add_system(System::<(Mut<Position>, Mut<Velocity>)>::new(
            |(position, velocity)| {
                if (position.0 + velocity.0 > 100f32) || (position.0 + velocity.0 < 0f32) {
                    velocity.0 = -velocity.0;
                }

                if (position.1 + velocity.1 > 100f32) || (position.1 + velocity.1 < 0f32) {
                    velocity.1 = -velocity.1;
                }

                position.0 += velocity.0;
                position.1 += velocity.1;
            },
        ));
    }
}

fn main() {
    Engine::new("fjorgyn").ignite(Box::new(GameState)).unwrap();
}
