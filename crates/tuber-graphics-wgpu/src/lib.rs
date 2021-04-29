use crate::rectangle_renderer::RectangleRenderer;
use crate::sprite_renderer::SpriteRenderer;
use crate::texture::Texture;
use futures;
use std::collections::HashMap;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::query::accessors::R;
use tuber_ecs::system::SystemBundle;
use tuber_graphics::texture::TextureData;
use tuber_graphics::{
    Graphics, GraphicsAPI, GraphicsError, RectangleShape, Sprite, Transform2D, Window, WindowSize,
};

mod rectangle_renderer;
mod sprite_renderer;
mod texture;

#[derive(Debug)]
pub enum TuberGraphicsWGPUError {}

pub struct GraphicsWGPU {
    wgpu_state: Option<WGPUState>,
    textures: HashMap<String, Texture>,
}

pub struct WGPUState {
    _surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    _sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    _window_size: WindowSize,
    rectangle_renderer: RectangleRenderer,
    sprite_renderer: SpriteRenderer,
}

impl GraphicsWGPU {
    pub fn new() -> Self {
        Self {
            wgpu_state: None,
            textures: HashMap::new(),
        }
    }
}

impl GraphicsAPI for GraphicsWGPU {
    fn initialize(&mut self, window: Window, window_size: WindowSize) {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = async {
            instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                })
                .await
        };
        let adapter = futures::executor::block_on(adapter).unwrap();

        let device_and_queue = async {
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: wgpu::Features::empty(),
                        limits: wgpu::Limits::default(),
                        label: None,
                    },
                    None,
                )
                .await
        };
        let (device, queue) = futures::executor::block_on(device_and_queue).unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: window_size.0,
            height: window_size.1,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let format = sc_desc.format;

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let rectangle_renderer = RectangleRenderer::new(&device, &format);
        let sprite_renderer = SpriteRenderer::new(&device, &queue, &format);

        self.wgpu_state = Some(WGPUState {
            _surface: surface,
            device,
            queue,
            _sc_desc: sc_desc,
            swap_chain,
            _window_size: window_size,
            rectangle_renderer,
            sprite_renderer,
        });
    }

    fn default_system_bundle(&mut self) -> SystemBundle {
        let mut bundle = SystemBundle::new();
        bundle.add_system(render);
        bundle
    }

    fn render(&mut self) {
        let state = self.wgpu_state.as_mut().expect("Graphics is uninitialized");
        let frame = state.swap_chain.get_current_frame().unwrap().output;
        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            state.rectangle_renderer.render(&mut render_pass);
            state.sprite_renderer.render(&mut render_pass);
        }

        state.queue.submit(std::iter::once(encoder.finish()));
    }

    fn prepare_rectangle(&mut self, rectangle: &RectangleShape, transform: &Transform2D) {
        let state = self.wgpu_state.as_mut().unwrap();
        state
            .rectangle_renderer
            .prepare(&state.queue, rectangle, transform);
    }

    fn prepare_sprite(
        &mut self,
        sprite: &Sprite,
        transform: &Transform2D,
    ) -> Result<(), GraphicsError> {
        let state = self.wgpu_state.as_mut().unwrap();
        state.sprite_renderer.prepare(
            &state.device,
            &state.queue,
            sprite,
            transform,
            &self.textures,
        );
        Ok(())
    }

    fn finish_prepare_render(&mut self) {}

    fn is_texture_in_memory(&self, texture_identifier: &str) -> bool {
        self.textures.contains_key(texture_identifier)
    }

    fn load_texture(&mut self, texture_data: TextureData) {
        let state = self.wgpu_state.as_ref().expect("Graphics is uninitialized");
        let identifier = texture_data.identifier.clone();
        let texture =
            Texture::from_texture_data(&state.device, &state.queue, texture_data).unwrap();
        self.textures.insert(identifier, texture);
    }
}

fn render(ecs: &mut Ecs) {
    let mut graphics = ecs.resource_mut::<Graphics>();
    for (_, (rectangle_shape, transform)) in ecs.query::<(R<RectangleShape>, R<Transform2D>)>() {
        graphics.prepare_rectangle(&rectangle_shape, &transform);
    }
    for (_, (sprite, transform)) in ecs.query::<(R<Sprite>, R<Transform2D>)>() {
        if let Err(e) = graphics.prepare_sprite(&sprite, &transform) {
            println!("{:?}", e);
        }
    }
    graphics.finish_prepare_render();
    graphics.render();
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float3,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float2,
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                },
            ],
        }
    }
}
