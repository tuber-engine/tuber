use crate::graphics::{SceneRenderer, SquareShape};
use crate::Position;
use async_trait::async_trait;
use futures::TryFutureExt;
use shaderc;
use tecs::core::Ecs;
use tecs::query::Imm;
use tecs::query::Queryable;
use wgpu::{BufferUsage, PresentMode, TextureFormat};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct WGPURenderer {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    quad_buffer: QuadBuffer,
    size: PhysicalSize<u32>,
}
impl WGPURenderer {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let surface = wgpu::Surface::create(window);
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: Default::default(),
            })
            .await;

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let vs_src = include_str!("../shaders/quad.vert");
        let fs_src = include_str!("../shaders/quad.frag");

        let mut compiler = shaderc::Compiler::new().unwrap();
        let vs_spirv = compiler
            .compile_into_spirv(
                vs_src,
                shaderc::ShaderKind::Vertex,
                "quad.vert",
                "main",
                None,
            )
            .unwrap();
        let fs_spirv = compiler
            .compile_into_spirv(
                fs_src,
                shaderc::ShaderKind::Fragment,
                "quad.frag",
                "main",
                None,
            )
            .unwrap();

        let vs_data = wgpu::read_spirv(std::io::Cursor::new(vs_spirv.as_binary_u8())).unwrap();
        let fs_data = wgpu::read_spirv(std::io::Cursor::new(fs_spirv.as_binary_u8())).unwrap();

        let vs_module = device.create_shader_module(&vs_data);
        let fs_module = device.create_shader_module(&fs_data);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let mut quad_buffer = QuadBuffer::new();
        quad_buffer.add_quad(Quad {
            top_left: Vertex {
                position: [0.0, 0.0, 0.0],
                color: [1.0, 0.0, 0.0],
            },
            bottom_left: Vertex {
                position: [1.0, 0.0, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            top_right: Vertex {
                position: [0.0, 1.0, 0.0],
                color: [0.0, 0.0, 1.0],
            },
            bottom_right: Vertex {
                position: [1.0, 1.0, 0.0],
                color: [0.0, 0.0, 0.0],
            },
        });
        quad_buffer.add_quad(Quad {
            top_left: Vertex {
                position: [0.0, 0.0, 0.0],
                color: [1.0, 0.0, 0.0],
            },
            bottom_left: Vertex {
                position: [-1.0, 0.0, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            top_right: Vertex {
                position: [0.0, -1.0, 0.0],
                color: [0.0, 0.0, 1.0],
            },
            bottom_right: Vertex {
                position: [-1.0, -1.0, 0.0],
                color: [0.0, 0.0, 0.0],
            },
        });

        let vertex_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(quad_buffer.quads()),
            wgpu::BufferUsage::VERTEX,
        );
        let vertex_count = quad_buffer.quad_count() as u32 * 4u32;
        let index_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(quad_buffer.indices()),
            wgpu::BufferUsage::INDEX,
        );
        let index_count = quad_buffer.index_count() as u32;

        Self {
            surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
            render_pipeline,
            vertex_buffer,
            vertex_count,
            index_buffer,
            index_count,
            quad_buffer,
            size,
        }
    }

    fn process_queue(&mut self) {
        let frame = self
            .swap_chain
            .get_next_texture()
            .expect("Timeout getting texture");
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::WHITE,
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
            render_pass.set_index_buffer(&self.index_buffer, 0, 0);
            render_pass.draw_indexed(0..self.index_count as u32, 0, 0..1);
        }

        self.queue.submit(&[encoder.finish()]);
    }
}

impl SceneRenderer for WGPURenderer {
    fn render(&mut self) {
        self.process_queue();
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Quad {
    top_left: Vertex,
    bottom_left: Vertex,
    top_right: Vertex,
    bottom_right: Vertex,
}

unsafe impl bytemuck::Pod for Quad {}
unsafe impl bytemuck::Zeroable for Quad {}

struct QuadBuffer {
    quads: Vec<Quad>,
    indices: Vec<u16>,
}

impl QuadBuffer {
    pub fn new() -> Self {
        QuadBuffer {
            quads: vec![],
            indices: vec![],
        }
    }

    pub fn add_quad(&mut self, quad: Quad) {
        self.quads.push(quad);

        let last_index = if let Some(index) = self.indices.last() {
            *index + 1
        } else {
            0
        };

        self.indices.append(&mut vec![
            last_index + 0,
            last_index + 1,
            last_index + 2,
            last_index + 2,
            last_index + 1,
            last_index + 3,
        ])
    }

    pub fn quads(&self) -> &[Quad] {
        self.quads.as_slice()
    }

    pub fn indices(&self) -> &[u16] {
        self.indices.as_slice()
    }

    pub fn quad_count(&self) -> usize {
        self.quads.len()
    }

    pub fn index_count(&self) -> usize {
        self.indices.len()
    }
}
