/*
* MIT License
*
* Copyright (c) 2020 Tuber contributors
*
* Permission is hereby granted, free of charge, to any person obtaining a copy
* of this software and associated documentation files (the "Software"), to deal
* in the Software without restriction, including without limitation the rights
* to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
* copies of the Software, and to permit persons to whom the Software is
* furnished to do so, subject to the following conditions:
*
* The above copyright notice and this permission notice shall be included in all
* copies or substantial portions of the Software.
*
* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
* FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
* AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
* LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
* OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
* SOFTWARE.
*/
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub struct Engine {
    application_title: String,
}
impl Engine {
    pub fn new(application_title: &str) -> Engine {
        Engine {
            application_title: application_title.into(),
        }
    }

    pub fn ignite(
        &mut self,
        initial_game_state: Box<dyn GameState>,
    ) -> Result<(), winit::error::OsError> {
        let mut game_state_stack = GameStateStack::new();
        game_state_stack.push(initial_game_state);
        game_state_stack
            .current_state()
            .expect("No game state on stack")
            .initialize();

        let event_loop = EventLoop::new();
        let _window = WindowBuilder::new()
            .with_title(&self.application_title)
            .build(&event_loop)?;

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::RedrawEventsCleared => {
                if let Some(state) = game_state_stack.current_state() {
                    state.update();
                }
            }
            _ => *control_flow = ControlFlow::Poll,
        });
    }
}

pub struct GameStateStack {
    game_states: Vec<Box<dyn GameState>>,
}

impl GameStateStack {
    pub fn new() -> GameStateStack {
        GameStateStack {
            game_states: vec![],
        }
    }

    pub fn push(&mut self, state: Box<dyn GameState>) {
        self.game_states.push(state);
    }

    pub fn pop(&mut self) {
        self.game_states.pop();
    }

    pub fn current_state(&mut self) -> Option<&mut Box<dyn GameState>> {
        self.game_states.last_mut()
    }
}

pub trait GameState {
    fn initialize(&mut self);
    fn update(&mut self);
}
