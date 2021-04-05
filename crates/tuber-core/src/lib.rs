use tuber_ecs::ecs::Ecs;
use tuber_ecs::system::SystemBundle;

pub struct Engine {
    ecs: Ecs,
    system_bundles: Vec<SystemBundle>,
}

impl Engine {
    pub fn new() -> Engine {
        Self {
            ecs: Ecs::new(),
            system_bundles: vec![],
        }
    }

    pub fn ecs(&mut self) -> &mut Ecs {
        &mut self.ecs
    }

    pub fn add_system_bundle(&mut self, system_bundle: SystemBundle) {
        self.system_bundles.push(system_bundle);
    }

    pub fn step(&mut self) {
        for bundle in &mut self.system_bundles {
            bundle.step(&mut self.ecs);
        }
    }

    pub fn ignite(mut self) -> Result<()> {
        loop {
            self.step();
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {}

pub trait TuberRunner {
    fn run(&mut self, engine: Engine) -> Result<()>;
}
