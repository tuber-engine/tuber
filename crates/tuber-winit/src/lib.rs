use tuber_core::{Engine, Result, TuberRunner};
use tuber_graphics::{Graphics, GraphicsAPI, Window};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct WinitTuberRunner;
impl TuberRunner for WinitTuberRunner {
    fn run(&mut self, mut engine: Engine, mut graphics: Graphics) -> Result<()> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        graphics.initialize(
            Window(Box::new(
                &window as &dyn raw_window_handle::HasRawWindowHandle,
            )),
            (window.inner_size().width, window.inner_size().height),
        );
        engine.ecs().insert_resource(graphics);

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => *control_flow = ControlFlow::Exit,
                Event::MainEventsCleared => {
                    engine.step();
                }
                _ => (),
            }
        })
    }
}
