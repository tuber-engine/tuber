use crate::rectangle_renderer::RectangleRenderer;
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
use wgpu::util::DeviceExt;
use wgpu::{BlendFactor, BlendOperation, FragmentState, VertexState};

mod rectangle_renderer;
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
    colored_render_pipeline: wgpu::RenderPipeline,
    default_texture: Texture,
    textured_render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    _index_buffer: wgpu::Buffer,
    num_vertices: u32,
    pending_vertices: Vec<Vertex>,
    rectangle_renderer: RectangleRenderer,
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

        let colored_vertex_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/colored_shader.vert.spv"));
        let colored_fragment_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/colored_shader.frag.spv"));
        let textured_vertex_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/textured_shader.vert.spv"));
        let textured_fragment_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/textured_shader.frag.spv"));

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
            });
        let default_texture_data = TextureData::from_bytes(
            "default_texture.png",
            include_bytes!("textures/default_texture.png"),
        )
        .unwrap();
        let default_texture =
            Texture::from_texture_data(&device, &queue, default_texture_data).unwrap();

        let colored_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Colored Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let colored_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Colored Render Pipeline"),
                layout: Some(&colored_render_pipeline_layout),
                vertex: VertexState {
                    module: &colored_vertex_shader_module,
                    entry_point: "main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(FragmentState {
                    module: &colored_fragment_shader_module,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: sc_desc.format.clone(),
                        alpha_blend: wgpu::BlendState::REPLACE,
                        color_blend: wgpu::BlendState::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::Back,
                    polygon_mode: wgpu::PolygonMode::Fill,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            });

        let textured_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let textured_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&textured_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &textured_vertex_shader_module,
                    entry_point: "main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &textured_fragment_shader_module,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: sc_desc.format,
                        alpha_blend: wgpu::BlendState {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        color_blend: wgpu::BlendState::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::Back,
                    polygon_mode: wgpu::PolygonMode::Fill,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            usage: wgpu::BufferUsage::VERTEX,
            size: 0,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            usage: wgpu::BufferUsage::INDEX,
            size: 0,
            mapped_at_creation: false,
        });

        let rectangle_renderer = RectangleRenderer::new(&device, format);

        let num_vertices = 0;

        self.wgpu_state = Some(WGPUState {
            _surface: surface,
            device,
            queue,
            _sc_desc: sc_desc,
            swap_chain,
            _window_size: window_size,
            colored_render_pipeline,
            default_texture,
            textured_render_pipeline,
            vertex_buffer,
            _index_buffer: index_buffer,
            num_vertices,
            pending_vertices: vec![],
            rectangle_renderer,
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
        }

        state.queue.submit(std::iter::once(encoder.finish()));
    }

    fn prepare_rectangle(&mut self, rectangle: &RectangleShape, transform: &Transform2D) {
        let mut state = self.wgpu_state.as_mut().unwrap();
        state
            .rectangle_renderer
            .prepare(&state.queue, rectangle, transform);
    }

    fn prepare_sprite(
        &mut self,
        sprite: &Sprite,
        transform: &Transform2D,
    ) -> Result<(), GraphicsError> {
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

struct Quad {
    texture: Option<String>,
    vertices: Vec<Vertex>,
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
