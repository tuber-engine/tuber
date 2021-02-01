use winit::{event::{Event, WindowEvent}, event_loop::{EventLoop, ControlFlow}, window::{WindowBuilder, Window}};

pub struct Engine;

impl Engine {
    pub fn new() -> Engine {
        Self
    }

    pub fn ignite(&mut self) -> Result<()> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop)?;
        
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit
                },
                _ => ()
            }
        });
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    WinitOsError(winit::error::OsError)
}

impl From<winit::error::OsError> for Error {
fn from(error: winit::error::OsError) -> Self { Error::WinitOsError(error) }
}
