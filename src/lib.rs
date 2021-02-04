use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct Engine {
    state_stack: Vec<GameState>,
}

impl Engine {
    pub fn new() -> Engine {
        Self {
            state_stack: vec![],
        }
    }

    pub fn ignite(mut self) -> Result<()> {
        let event_loop = EventLoop::new();
        let _window = WindowBuilder::new().build(&event_loop)?;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                Event::MainEventsCleared => {
                    self.current_state().step();
                }
                _ => (),
            }
        });
    }

    pub fn push_state(&mut self, state: GameState) {
        self.state_stack.push(state);
    }

    fn current_state(&mut self) -> &mut GameState {
        self.state_stack.last_mut().expect("No state")
    }
}

pub struct GameState {
    system_bundles: Vec<SystemBundle>,
}

impl GameState {
    pub fn step(&mut self) {
        let bundles = &self.system_bundles;
        for system_bundle in bundles.iter() {
            for system in system_bundle.system_functions.iter() {
                (system)()
            }
        }
    }
}

pub struct GameStateBuilder {
    system_bundles: Vec<SystemBundle>,
}

impl GameStateBuilder {
    pub fn new() -> Self {
        GameStateBuilder {
            system_bundles: vec![],
        }
    }

    pub fn with_system_bundle(mut self, system_bundle: SystemBundle) -> Self {
        self.system_bundles.push(system_bundle);
        self
    }

    pub fn build(self) -> GameState {
        GameState {
            system_bundles: self.system_bundles,
        }
    }
}

pub struct SystemBundle {
    system_functions: Vec<SystemFunction>,
}

type SystemFunction = Box<dyn Fn()>;

pub struct SystemBundleBuilder {
    systems: Vec<SystemFunction>,
}

impl SystemBundleBuilder {
    pub fn new() -> Self {
        SystemBundleBuilder { systems: vec![] }
    }

    pub fn with_system(mut self, system: Box<dyn Fn()>) -> Self {
        self.systems.push(system.into());
        self
    }

    pub fn build(self) -> SystemBundle {
        SystemBundle {
            system_functions: self.systems,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    WinitOsError(winit::error::OsError),
}

impl From<winit::error::OsError> for Error {
    fn from(error: winit::error::OsError) -> Self {
        Error::WinitOsError(error)
    }
}
