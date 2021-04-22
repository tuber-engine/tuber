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

mod texture;

#[derive(Debug)]
pub enum TuberGraphicsWGPUError {}

pub struct GraphicsWGPU {
    wgpu_state: Option<WGPUState>,
    textures: HashMap<String, Texture>,
    pending_quads: Vec<Quad>,
}

pub struct WGPUState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    window_size: WindowSize,
    colored_render_pipeline: wgpu::RenderPipeline,
    default_texture: Texture,
    textured_render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_vertices: u32,

    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    pending_vertices: Vec<Vertex>,
}

impl GraphicsWGPU {
    pub fn new() -> Self {
        Self {
            wgpu_state: None,
            textures: HashMap::new(),
            pending_quads: vec![],
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
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let colored_vertex_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/colored_shader.vert.spv"));
        let colored_fragment_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/colored_shader.frag.spv"));
        let textured_vertex_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/textured_shader.vert.spv"));
        let textured_fragment_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/textured_shader.frag.spv"));

        let uniforms = Uniforms::new();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

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

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
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

        let texture_bind_group = &default_texture.bind_group;

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
                        format: sc_desc.format,
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

        let num_vertices = 0;

        self.wgpu_state = Some(WGPUState {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            window_size,
            colored_render_pipeline,
            default_texture,
            textured_render_pipeline,
            vertex_buffer,
            index_buffer,
            num_vertices,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            pending_vertices: vec![],
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
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            for (i, quad) in self.pending_quads.iter().enumerate() {
                match &quad.texture {
                    Some(texture) => {
                        let bind_group = if let Some(texture) = &self.textures.get(texture) {
                            &texture.bind_group
                        } else {
                            &state.default_texture.bind_group
                        };

                        render_pass.set_pipeline(&state.textured_render_pipeline);
                        render_pass.set_bind_group(0, &state.uniform_bind_group, &[]);
                        render_pass.set_bind_group(1, &bind_group, &[]);
                    }
                    None => {
                        render_pass.set_pipeline(&state.colored_render_pipeline);
                        render_pass.set_bind_group(0, &state.uniform_bind_group, &[]);
                    }
                }
                render_pass.set_vertex_buffer(0, state.vertex_buffer.slice(..));
                render_pass.draw(i as u32 * 6..i as u32 * 6 + 6, 0..1);
            }
        }

        state.queue.submit(std::iter::once(encoder.finish()));
        self.pending_quads.clear();
    }

    fn prepare_rectangle(&mut self, rectangle_shape: &RectangleShape, transform: &Transform2D) {
        self.pending_quads.push(Quad {
            texture: None,
            vertices: vec![
                Vertex {
                    position: [transform.translation.0, transform.translation.1, 0.0],
                    color: [
                        rectangle_shape.color.0,
                        rectangle_shape.color.1,
                        rectangle_shape.color.1,
                    ],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [
                        transform.translation.0,
                        transform.translation.1 + rectangle_shape.height,
                        0.0,
                    ],
                    color: [
                        rectangle_shape.color.0,
                        rectangle_shape.color.1,
                        rectangle_shape.color.1,
                    ],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [
                        transform.translation.0 + rectangle_shape.width,
                        transform.translation.1,
                        0.0,
                    ],
                    color: [
                        rectangle_shape.color.0,
                        rectangle_shape.color.1,
                        rectangle_shape.color.1,
                    ],
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    position: [
                        transform.translation.0 + rectangle_shape.width,
                        transform.translation.1,
                        0.0,
                    ],
                    color: [
                        rectangle_shape.color.0,
                        rectangle_shape.color.1,
                        rectangle_shape.color.1,
                    ],
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    position: [
                        transform.translation.0,
                        transform.translation.1 + rectangle_shape.height,
                        0.0,
                    ],
                    color: [
                        rectangle_shape.color.0,
                        rectangle_shape.color.1,
                        rectangle_shape.color.1,
                    ],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [
                        transform.translation.0 + rectangle_shape.width,
                        transform.translation.1 + rectangle_shape.height,
                        0.0,
                    ],
                    color: [
                        rectangle_shape.color.0,
                        rectangle_shape.color.1,
                        rectangle_shape.color.1,
                    ],
                    tex_coords: [1.0, 1.0],
                },
            ],
        });
    }

    fn prepare_sprite(
        &mut self,
        sprite: &Sprite,
        transform: &Transform2D,
    ) -> Result<(), GraphicsError> {
        self.pending_quads.push(Quad {
            texture: Some(sprite.texture.clone()),
            vertices: vec![
                Vertex {
                    position: [transform.translation.0, transform.translation.1, 0.0],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [
                        transform.translation.0,
                        transform.translation.1 + sprite.height,
                        0.0,
                    ],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [
                        transform.translation.0 + sprite.width,
                        transform.translation.1,
                        0.0,
                    ],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    position: [
                        transform.translation.0 + sprite.width,
                        transform.translation.1,
                        0.0,
                    ],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    position: [
                        transform.translation.0,
                        transform.translation.1 + sprite.height,
                        0.0,
                    ],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [
                        transform.translation.0 + sprite.width,
                        transform.translation.1 + sprite.height,
                        0.0,
                    ],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [1.0, 1.0],
                },
            ],
        });
        Ok(())
    }

    fn finish_prepare_render(&mut self) {
        let mut state = self.wgpu_state.as_mut().expect("Graphics is uninitialized");
        if self.pending_quads.len() > 0 {
            let vertices: Vec<Vertex> = self
                .pending_quads
                .iter()
                .map(|q| q.vertices.clone())
                .flat_map(|v| v)
                .collect();
            let new_buffer = state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_SRC,
                });
            state.vertex_buffer = new_buffer;
        } else {
            let new_buffer = state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(state.pending_vertices.as_slice()),
                    usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_SRC,
                });
            state.vertex_buffer = new_buffer;
            state.num_vertices = state.pending_vertices.len() as u32;
            state.pending_vertices.clear();
        }
    }

    fn is_texture_in_memory(&self, texture_identifier: &str) -> bool {
        let state = self.wgpu_state.as_ref().expect("Graphics is uninitialized");
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

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            view_proj: cgmath::ortho(0.0, 800.0, 600.0, 0.0, -100.0, 100.0).into(),
        }
    }
}
