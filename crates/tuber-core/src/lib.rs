use crate::input::InputState;
use ecs::ecs::Ecs;
use ecs::system::SystemBundle;
pub use tuber_ecs as ecs;
use tuber_graphics::Graphics;

pub mod input;

pub struct DeltaTime(f64);

pub struct Engine {
    ecs: Ecs,
    system_bundles: Vec<SystemBundle>,
}

impl Engine {
    pub fn new() -> Engine {
        let mut ecs = Ecs::new();
        ecs.insert_resource(InputState::new());
        Self {
            ecs,
            system_bundles: vec![],
        }
    }

    pub fn handle_input(&mut self, input: input::Input) {
        let mut input_state = self.ecs.resource_mut::<InputState>();
        input_state.handle_input(input);
    }

    pub fn ecs(&mut self) -> &mut Ecs {
        &mut self.ecs
    }

    pub fn add_system_bundle(&mut self, system_bundle: SystemBundle) {
        self.system_bundles.push(system_bundle);
    }

    pub fn step(&mut self, delta_time: f64) {
        self.ecs.insert_resource(DeltaTime(delta_time));
        for bundle in &mut self.system_bundles {
            bundle.step(&mut self.ecs);
        }
    }

    pub fn ignite(mut self) -> Result<()> {
        loop {
            self.step(1.0);
        }
    }
}

pub trait TuberRunner {
    fn run(&mut self, engine: Engine, graphics: Graphics) -> Result<()>;
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {}
