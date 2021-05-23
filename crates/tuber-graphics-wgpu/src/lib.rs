use crate::bounding_box_renderer::BoundingBoxRenderer;
use crate::quad_renderer::QuadRenderer;
use crate::texture::Texture;
use futures;
use std::collections::HashMap;
use tuber_graphics::camera::OrthographicCamera;
use tuber_graphics::texture::TextureData;
use tuber_graphics::{LowLevelGraphicsAPI, QuadDescription, Transform2D, Window, WindowSize};

mod bounding_box_renderer;
mod quad_renderer;
mod texture;

#[derive(Debug)]
pub enum TuberGraphicsWGPUError {}

pub struct GraphicsWGPU {
    wgpu_state: Option<WGPUState>,
    textures: HashMap<String, Texture>,
    camera_id: Option<usize>,
}

pub struct WGPUState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    window_size: WindowSize,
    quad_renderer: QuadRenderer,
    bounding_box_renderer: BoundingBoxRenderer,
}

impl GraphicsWGPU {
    pub fn new() -> Self {
        Self {
            wgpu_state: None,
            textures: HashMap::new(),
            camera_id: None,
        }
    }
}

impl LowLevelGraphicsAPI for GraphicsWGPU {
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
            present_mode: wgpu::PresentMode::Immediate,
        };
        let format = sc_desc.format;

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        let quad_renderer = QuadRenderer::new(&device, &queue, &format);
        let bounding_box_renderer = BoundingBoxRenderer::new(&device, &format);

        self.wgpu_state = Some(WGPUState {
            surface: surface,
            device,
            queue,
            sc_desc: sc_desc,
            swap_chain,
            window_size,
            quad_renderer,
            bounding_box_renderer,
        });
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

            state.quad_renderer.render(&mut render_pass);
            state.bounding_box_renderer.render(&mut render_pass);
        }

        state.queue.submit(std::iter::once(encoder.finish()));
    }

    fn prepare_quad(
        &mut self,
        quad_description: &QuadDescription,
        transform: &Transform2D,
        bounding_box_rendering: bool,
    ) {
        let state = self.wgpu_state.as_mut().expect("Graphics is uninitialized");
        state.quad_renderer.prepare(
            &state.device,
            &state.queue,
            quad_description,
            transform,
            &self.textures,
        );

        if bounding_box_rendering {
            state.bounding_box_renderer.prepare(
                &state.queue,
                quad_description.width,
                quad_description.height,
                transform,
            );
        }
    }

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

    fn update_camera(
        &mut self,
        camera_id: usize,
        camera: &OrthographicCamera,
        transform: &Transform2D,
    ) {
        let state = self.wgpu_state.as_mut().expect("Graphics is uninitialized");
        self.camera_id = Some(camera_id);
        state
            .quad_renderer
            .set_camera(&state.queue, camera, transform);
        state
            .bounding_box_renderer
            .set_camera(&state.queue, camera, transform);
    }

    fn on_window_resized(&mut self, new_size: WindowSize) {
        let state = self.wgpu_state.as_mut().expect("Graphics is uninitialized");
        state.window_size = new_size;
        state.sc_desc.width = new_size.0;
        state.sc_desc.height = new_size.1;
        state.swap_chain = state
            .device
            .create_swap_chain(&state.surface, &state.sc_desc);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
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
