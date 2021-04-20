use tuber_core::input::keyboard::Key;
use tuber_core::input::Input;
use tuber_core::{Engine, Result, TuberRunner};
use tuber_graphics::{Graphics, GraphicsAPI, Window};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
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
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { input, .. },
                    window_id,
                } if window_id == window.id() => {
                    engine.handle_input(KeyboardInputWrapper(input).into());
                }
                Event::MainEventsCleared => {
                    engine.step();
                }
                _ => (),
            }
        })
    }
}

struct KeyboardInputWrapper(KeyboardInput);
impl Into<Input> for KeyboardInputWrapper {
    fn into(self) -> Input {
        match self.0 {
            KeyboardInput {
                virtual_keycode,
                state,
                ..
            } if state == ElementState::Pressed => {
                Input::KeyDown(VirtualKeyCodeWrapper(virtual_keycode.unwrap()).into())
            }
            KeyboardInput {
                virtual_keycode,
                state,
                ..
            } if state == ElementState::Released => {
                Input::KeyUp(VirtualKeyCodeWrapper(virtual_keycode.unwrap()).into())
            }
            _ => panic!("unknown input"),
        }
    }
}

struct VirtualKeyCodeWrapper(VirtualKeyCode);
impl Into<Key> for VirtualKeyCodeWrapper {
    fn into(self) -> Key {
        match self.0 {
            VirtualKeyCode::A => Key::A,
            VirtualKeyCode::B => Key::B,
            VirtualKeyCode::C => Key::C,
            VirtualKeyCode::D => Key::D,
            VirtualKeyCode::E => Key::E,
            VirtualKeyCode::F => Key::F,
            VirtualKeyCode::G => Key::G,
            VirtualKeyCode::H => Key::H,
            VirtualKeyCode::I => Key::I,
            VirtualKeyCode::J => Key::J,
            VirtualKeyCode::K => Key::K,
            VirtualKeyCode::L => Key::L,
            VirtualKeyCode::M => Key::M,
            VirtualKeyCode::N => Key::N,
            VirtualKeyCode::O => Key::O,
            VirtualKeyCode::P => Key::P,
            VirtualKeyCode::Q => Key::Q,
            VirtualKeyCode::R => Key::R,
            VirtualKeyCode::S => Key::S,
            VirtualKeyCode::T => Key::T,
            VirtualKeyCode::U => Key::U,
            VirtualKeyCode::V => Key::V,
            VirtualKeyCode::W => Key::W,
            VirtualKeyCode::X => Key::X,
            VirtualKeyCode::Y => Key::Y,
            VirtualKeyCode::Z => Key::Z,
            VirtualKeyCode::Key0 => Key::Number0,
            VirtualKeyCode::Key1 => Key::Number1,
            VirtualKeyCode::Key2 => Key::Number2,
            VirtualKeyCode::Key3 => Key::Number3,
            VirtualKeyCode::Key4 => Key::Number4,
            VirtualKeyCode::Key5 => Key::Number5,
            VirtualKeyCode::Key6 => Key::Number6,
            VirtualKeyCode::Key7 => Key::Number7,
            VirtualKeyCode::Key8 => Key::Number8,
            VirtualKeyCode::Key9 => Key::Number9,
            VirtualKeyCode::Space => Key::Spacebar,
            VirtualKeyCode::Return => Key::Return,
            VirtualKeyCode::LShift => Key::LShift,
            VirtualKeyCode::RShift => Key::RShift,
            VirtualKeyCode::LControl => Key::LControl,
            VirtualKeyCode::RControl => Key::RControl,
            VirtualKeyCode::Escape => Key::Escape,
            _ => panic!("Unknown key"),
        }
    }
}
