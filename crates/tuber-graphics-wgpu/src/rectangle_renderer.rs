use crate::Vertex;
use cgmath::Vector2;
use tuber_graphics::{RectangleShape, Transform2D};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferDescriptor, Device, FragmentState, RenderPass, TextureFormat};

const MAX_INSTANCE_COUNT: u64 = 100_000;
const VERTEX_COUNT_PER_INSTANCE: u32 = 6;
const INSTANCE_BUFFER_SIZE: u64 = MAX_INSTANCE_COUNT * std::mem::size_of::<InstanceRaw>() as u64;

pub(crate) struct RectangleRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    _uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instances: Vec<Instance>,
}
impl RectangleRenderer {
    pub fn new(device: &Device, texture_format: &TextureFormat) -> Self {
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
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(FragmentState {
                module: &colored_fragment_shader_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: *texture_format,
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

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("rectangle_renderer_vertex_buffer"),
            contents: bytemuck::cast_slice(&[
                Vertex {
                    position: [0.0, 0.0, 0.0],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [0.0, 1.0, 0.0],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [1.0, 0.0, 0.0],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    position: [1.0, 0.0, 0.0],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    position: [0.0, 1.0, 0.0],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [1.0, 1.0, 0.0],
                    color: [1.0, 1.0, 1.0],
                    tex_coords: [1.0, 1.0],
                },
            ]),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("rectangle_renderer_instance_buffer"),
            size: INSTANCE_BUFFER_SIZE,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            uniform_bind_group,
            _uniform_buffer: uniform_buffer,
            vertex_buffer,
            instance_buffer,
            instances: vec![],
        }
    }

    pub fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        rectangle_shape: &RectangleShape,
        transform_2d: &Transform2D,
    ) {
        let instance = Instance {
            model: (*transform_2d).into(),
            size: Vector2 {
                x: rectangle_shape.width,
                y: rectangle_shape.height,
            },
            color: rectangle_shape.color,
        };

        queue.write_buffer(
            &self.instance_buffer,
            self.instances.len() as u64 * std::mem::size_of::<InstanceRaw>() as u64,
            bytemuck::cast_slice(&[instance.to_raw()]),
        );
        self.instances.push(instance);
    }

    pub fn render<'rpass>(&'rpass mut self, render_pass: &mut RenderPass<'rpass>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw(0..VERTEX_COUNT_PER_INSTANCE, 0..self.instances.len() as _);

        self.instances.clear();
    }
}

struct Instance {
    model: cgmath::Matrix4<f32>,
    size: cgmath::Vector2<f32>,
    color: (f32, f32, f32),
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: self.model.into(),
            color: [self.color.0, self.color.1, self.color.2],
            size: [self.size.x, self.size.y],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
    color: [f32; 3],
    size: [f32; 2],
}

impl InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float4,
                    offset: 0,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float4,
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float4,
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float4,
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 6,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float3,
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 7,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float2,
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 8,
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
