use crate::Vertex;
use tuber_graphics::{RectangleShape, Transform2D};
use wgpu::util::DeviceExt;
use wgpu::{BufferDescriptor, Device, FragmentState, RenderPass, TextureFormat};

const VERTEX_BUFFER_SIZE: u64 = 1024u64 * std::mem::size_of::<Vertex>() as u64;

pub(crate) struct RectangleRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
}
impl RectangleRenderer {
    pub fn new(device: &Device, texture_format: TextureFormat) -> Self {
        let colored_vertex_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/colored_shader.vert.spv"));
        let colored_fragment_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/colored_shader.frag.spv"));

        let uniforms = Uniforms::new();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("rectangle_renderer_uniform_bind_group_layout"),
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
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rectangle_renderer_uniform_bind_group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rectangle_renderer_render_pipeline_layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rectangle_renderer_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &colored_vertex_shader_module,
                entry_point: "main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &colored_fragment_shader_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: texture_format,
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

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("rectangle_renderer_vertex_buffer"),
            size: VERTEX_BUFFER_SIZE,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            uniform_bind_group,
            uniform_buffer,
            vertex_buffer,
            vertex_count: 0,
        }
    }

    pub fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        rectangle_shape: &RectangleShape,
        transform: &Transform2D,
    ) {
        queue.write_buffer(
            &self.vertex_buffer,
            self.vertex_count as u64 * std::mem::size_of::<Vertex>() as u64,
            bytemuck::cast_slice(&[
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
            ]),
        );

        self.vertex_count += 6;
    }

    pub fn render<'rpass>(&'rpass mut self, render_pass: &mut RenderPass<'rpass>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);
        self.vertex_count = 0;
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
