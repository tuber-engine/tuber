use crate::graphics::{SceneRenderer, SquareShape};
use crate::Position;
use async_trait::async_trait;
use futures::TryFutureExt;
use shaderc;
use tecs::core::Ecs;
use tecs::query::Imm;
use tecs::query::Queryable;
use wgpu::{BufferDescriptor, BufferUsage, CommandEncoderDescriptor, PresentMode, TextureFormat};
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

        let mut quad_buffer = QuadBuffer::new(&device);
        quad_buffer.add_quad(
            Quad {
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
            },
            &device,
            &queue,
        );
        quad_buffer.add_quad(
            Quad {
                top_left: Vertex {
                    position: [-0.75, -0.25, 0.0],
                    color: [1.0, 0.0, 0.0],
                },
                bottom_left: Vertex {
                    position: [-0.75, -0.75, 0.0],
                    color: [0.0, 1.0, 0.0],
                },
                top_right: Vertex {
                    position: [-0.25, -0.25, 0.0],
                    color: [0.0, 0.0, 1.0],
                },
                bottom_right: Vertex {
                    position: [-0.25, -0.75, 0.0],
                    color: [1.0, 1.0, 1.0],
                },
            },
            &device,
            &queue,
        );

        Self {
            surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
            render_pipeline,
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
            render_pass.set_vertex_buffer(0, self.quad_buffer.vertex_buffer(), 0, 0);
            render_pass.set_index_buffer(self.quad_buffer.index_buffer(), 0, 0);
            render_pass.draw_indexed(0..12 as u32, 0, 0..1);
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
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    quad_count: usize,
}

impl QuadBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("quad buffer"),
            size: 4000,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });
        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("quad index buffer"),
            size: 1000,
            usage: wgpu::BufferUsage::INDEX | wgpu::BufferUsage::COPY_DST,
        });

        QuadBuffer {
            vertex_buffer,
            index_buffer,
            quad_count: 0,
        }
    }

    pub fn add_quad(&mut self, quad: Quad, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut command_encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Quad buffer command encoder"),
        });

        let vertices = device
            .create_buffer_with_data(bytemuck::cast_slice(&[quad]), wgpu::BufferUsage::COPY_SRC);

        let mut next_index = self.quad_count as u16 * 4;
        let indices = device.create_buffer_with_data(
            bytemuck::cast_slice(&[
                next_index + 0u16,
                next_index + 1u16,
                next_index + 2u16,
                next_index + 2u16,
                next_index + 1u16,
                next_index + 3u16,
            ]),
            wgpu::BufferUsage::COPY_SRC,
        );

        command_encoder.copy_buffer_to_buffer(
            &vertices,
            0,
            &self.vertex_buffer,
            self.quad_count as u64 * std::mem::size_of::<Quad>() as u64,
            std::mem::size_of::<Quad>() as u64,
        );

        command_encoder.copy_buffer_to_buffer(
            &indices,
            0,
            &self.index_buffer,
            self.quad_count as u64 * 6 * std::mem::size_of::<u16>() as u64,
            6 * std::mem::size_of::<u16>() as u64,
        );

        queue.submit(&[command_encoder.finish()]);
        self.quad_count += 1;
    }

    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    pub fn quad_count(&self) -> usize {
        self.quad_count
    }

    pub fn index_count(&self) -> usize {
        self.quad_count * 6
    }
}
