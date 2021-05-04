use crate::texture::Texture;
use crate::Vertex;
use cgmath::{Matrix4, Point2, Point3, SquareMatrix, Transform, Vector2, Vector3, Vector4};
use std::collections::HashMap;
use tuber_graphics::texture::TextureData;
use tuber_graphics::{QuadDescription, Transform2D};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroupLayout, BufferDescriptor, BufferUsage, Device, FragmentState, Queue, RenderPass,
    RenderPipeline, ShaderModule, TextureFormat,
};

const MAX_VERTEX_COUNT: u64 = 100_000;

pub(crate) struct BoundingBoxRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: usize,
    uniform_bind_group: wgpu::BindGroup,
    _uniform_buffer: wgpu::Buffer,
}

impl BoundingBoxRenderer {
    pub fn new(device: &Device, texture_format: &TextureFormat) -> Self {
        let uniforms = Uniforms::new();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad_renderer_uniform_buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("quad_renderer_uniform_bind_group_layout"),
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
            label: Some("quad_renderer_uniform_bind_group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline =
            Self::create_render_pipeline(&device, &uniform_bind_group_layout, texture_format);

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("bounding_box_renderer_vertex_buffer"),
            size: MAX_VERTEX_COUNT * std::mem::size_of::<Vertex>() as u64,
            usage: BufferUsage::VERTEX | BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            vertex_count: 0,
            uniform_bind_group,
            _uniform_buffer: uniform_buffer,
        }
    }

    fn create_render_pipeline(
        device: &Device,
        uniform_bind_group_layout: &BindGroupLayout,
        texture_format: &TextureFormat,
    ) -> RenderPipeline {
        let vertex_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/line_shader.vert.spv"));
        let fragment_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/line_shader.frag.spv"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("quad_renderer_colored_render_pipeline_layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad_renderer_colored_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader_module,
                entry_point: "main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &fragment_shader_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: *texture_format,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
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
        })
    }

    pub fn prepare(&mut self, queue: &Queue, width: f32, height: f32, transform_2d: &Transform2D) {
        let transform_matrix: Matrix4<f32> = transform_2d.clone().into();
        let top_left: Point3<f32> = transform_matrix.transform_point(Point3::new(0f32, 0f32, 0f32));
        let top_right: Point3<f32> =
            transform_matrix.transform_point(Point3::new(width, 0f32, 0f32));
        let bottom_left: Point3<f32> =
            transform_matrix.transform_point(Point3::new(0f32, height, 0f32));
        let bottom_right: Point3<f32> =
            transform_matrix.transform_point(Point3::new(width, height, 0f32));

        queue.write_buffer(
            &self.vertex_buffer,
            (self.vertex_count * std::mem::size_of::<Vertex>()) as u64,
            bytemuck::cast_slice(&[
                Vertex {
                    position: [bottom_left.x, bottom_left.y, 0.0],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [top_left.x, top_left.y, 0.0],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [top_left.x, top_left.y, 0.0],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [top_right.x, top_right.y, 0.0],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [top_right.x, top_right.y, 0.0],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [bottom_right.x, bottom_right.y, 0.0],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [bottom_right.x, bottom_right.y, 0.0],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [bottom_left.x, bottom_left.y, 0.0],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [0.0, 0.0],
                },
            ]),
        );

        self.vertex_count += 8;
    }

    pub fn render<'rpass>(&'rpass mut self, render_pass: &mut RenderPass<'rpass>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_count as u32, 0..1);
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
