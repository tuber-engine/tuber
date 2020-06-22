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
use crate::graphics::Renderer;
use tecs::{
    core::Ecs,
    query::Queryable,
    system::{Runnable, System},
};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

mod graphics;

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
        mut initial_state: Box<dyn State>,
    ) -> Result<(), winit::error::OsError> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(&self.application_title)
            .build(&event_loop)?;

        let mut renderer = futures::executor::block_on(Renderer::new(window));

        let mut ecs = Ecs::new();
        let mut system_schedule = SystemSchedule::new();

        initial_state.initialize(&mut ecs, &mut system_schedule);

        event_loop.run(move |event, _, control_flow| {
            *control_flow = winit::event_loop::ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                Event::RedrawEventsCleared => {
                    // Update game
                    system_schedule.run(&mut ecs);

                    // Execute renderer
                    renderer.render(&ecs);
                }
                _ => {}
            }
        });
    }
}

pub trait State {
    fn initialize(&mut self, ecs: &mut Ecs, system_schedule: &mut SystemSchedule);
}

pub struct SystemSchedule {
    systems: Vec<Box<dyn Runnable>>,
}

impl SystemSchedule {
    pub fn new() -> Self {
        Self { systems: vec![] }
    }

    pub fn add_system<Q: 'static + for<'a> Queryable<'a>>(&mut self, system: System<'static, Q>) {
        self.systems.push(Box::new(system))
    }

    pub fn run(&mut self, ecs: &mut Ecs) {
        for system in &mut self.systems {
            system.run(ecs);
        }
    }
}
