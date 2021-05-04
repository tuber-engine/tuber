use std::convert::{TryFrom, TryInto};
use tuber_core::input::keyboard::Key;
use tuber_core::input::Input;
use tuber_core::{Engine, Result as TuberResult, TuberRunner};
use tuber_graphics::{Graphics, Window};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

enum TuberWinitError {
    UknownVirtualKeycode(VirtualKeyCode),
    UknownKeyboardInput(KeyboardInput),
}

pub struct WinitTuberRunner;
impl TuberRunner for WinitTuberRunner {
    fn run(&mut self, mut engine: Engine, mut graphics: Graphics) -> TuberResult<()> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("tuber")
            .build(&event_loop)
            .unwrap();
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
                    if let Ok(input) = KeyboardInputWrapper(input).try_into() {
                        engine.handle_input(input);
                    }
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
impl TryFrom<KeyboardInputWrapper> for Input {
    type Error = TuberWinitError;

    fn try_from(input: KeyboardInputWrapper) -> Result<Self, Self::Error> {
        match input.0 {
            KeyboardInput {
                virtual_keycode,
                state,
                ..
            } if state == ElementState::Pressed && virtual_keycode.is_some() => Ok(Input::KeyDown(
                VirtualKeyCodeWrapper(virtual_keycode.unwrap()).try_into()?,
            )),
            KeyboardInput {
                virtual_keycode,
                state,
                ..
            } if state == ElementState::Released && virtual_keycode.is_some() => Ok(Input::KeyUp(
                VirtualKeyCodeWrapper(virtual_keycode.unwrap()).try_into()?,
            )),
            input => Err(TuberWinitError::UknownKeyboardInput(input)),
        }
    }
}

struct VirtualKeyCodeWrapper(VirtualKeyCode);
impl TryFrom<VirtualKeyCodeWrapper> for Key {
    type Error = TuberWinitError;

    fn try_from(value: VirtualKeyCodeWrapper) -> Result<Self, Self::Error> {
        match value.0 {
            VirtualKeyCode::A => Ok(Key::A),
            VirtualKeyCode::B => Ok(Key::B),
            VirtualKeyCode::C => Ok(Key::C),
            VirtualKeyCode::D => Ok(Key::D),
            VirtualKeyCode::E => Ok(Key::E),
            VirtualKeyCode::F => Ok(Key::F),
            VirtualKeyCode::G => Ok(Key::G),
            VirtualKeyCode::H => Ok(Key::H),
            VirtualKeyCode::I => Ok(Key::I),
            VirtualKeyCode::J => Ok(Key::J),
            VirtualKeyCode::K => Ok(Key::K),
            VirtualKeyCode::L => Ok(Key::L),
            VirtualKeyCode::M => Ok(Key::M),
            VirtualKeyCode::N => Ok(Key::N),
            VirtualKeyCode::O => Ok(Key::O),
            VirtualKeyCode::P => Ok(Key::P),
            VirtualKeyCode::Q => Ok(Key::Q),
            VirtualKeyCode::R => Ok(Key::R),
            VirtualKeyCode::S => Ok(Key::S),
            VirtualKeyCode::T => Ok(Key::T),
            VirtualKeyCode::U => Ok(Key::U),
            VirtualKeyCode::V => Ok(Key::V),
            VirtualKeyCode::W => Ok(Key::W),
            VirtualKeyCode::X => Ok(Key::X),
            VirtualKeyCode::Y => Ok(Key::Y),
            VirtualKeyCode::Z => Ok(Key::Z),
            VirtualKeyCode::Key0 => Ok(Key::Number0),
            VirtualKeyCode::Key1 => Ok(Key::Number1),
            VirtualKeyCode::Key2 => Ok(Key::Number2),
            VirtualKeyCode::Key3 => Ok(Key::Number3),
            VirtualKeyCode::Key4 => Ok(Key::Number4),
            VirtualKeyCode::Key5 => Ok(Key::Number5),
            VirtualKeyCode::Key6 => Ok(Key::Number6),
            VirtualKeyCode::Key7 => Ok(Key::Number7),
            VirtualKeyCode::Key8 => Ok(Key::Number8),
            VirtualKeyCode::Key9 => Ok(Key::Number9),
            VirtualKeyCode::Space => Ok(Key::Spacebar),
            VirtualKeyCode::Return => Ok(Key::Return),
            VirtualKeyCode::LShift => Ok(Key::LShift),
            VirtualKeyCode::RShift => Ok(Key::RShift),
            VirtualKeyCode::LControl => Ok(Key::LControl),
            VirtualKeyCode::RControl => Ok(Key::RControl),
            VirtualKeyCode::Escape => Ok(Key::Escape),
            virtual_keycode => Err(TuberWinitError::UknownVirtualKeycode(virtual_keycode)),
        }
    }
}
