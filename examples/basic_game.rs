use tecs::{
    core::Ecs,
    query::{Imm, Mut},
    system::System,
};
use tuber::{Engine, Position, State, SystemSchedule};

#[derive(Debug)]
struct Velocity(f32, f32);

struct GameState;
impl State for GameState {
    fn initialize(&mut self, ecs: &mut Ecs, systems: &mut SystemSchedule) {
        ecs.new_entity()
            .with_component(Position { x: 0f32, y: 0f32 })
            .with_component(Velocity(0.001f32, 0.002f32))
            .build();

        // Print system
        systems.add_system(System::<(Imm<Position>,)>::new(|(position,)| {
            println!("{:?}", position);
        }));
    }
}

fn main() {
    futures::executor::block_on(Engine::new("fjorgyn").ignite(Box::new(GameState))).unwrap();
}
