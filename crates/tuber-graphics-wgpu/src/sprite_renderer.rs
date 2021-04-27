use crate::texture::Texture;
use crate::Vertex;
use cgmath::Vector2;
use std::collections::HashMap;
use tuber_graphics::texture::TextureData;
use tuber_graphics::{Sprite, Transform2D};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroupDescriptor, BufferDescriptor, Device, FragmentState, Queue, RenderPass, TextureFormat,
};

const MAX_INSTANCE_COUNT: u64 = 100_000;
const VERTEX_COUNT_PER_INSTANCE: u64 = 6;
const INSTANCE_BUFFER_SIZE: u64 = MAX_INSTANCE_COUNT * std::mem::size_of::<InstanceRaw>() as u64;

pub(crate) struct SpriteRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    texture_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture: Texture,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_bind_groups: Vec<wgpu::BindGroup>,
    instances: Vec<Instance>,
}

impl SpriteRenderer {
    pub fn new(device: &Device, queue: &Queue, texture_format: &TextureFormat) -> Self {
        let textured_vertex_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/textured_shader.vert.spv"));
        let textured_fragment_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/textured_shader.frag.spv"));

        let uniforms = Uniforms::new();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite_renderer_uniform_buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let diffuse_bytes = include_bytes!("./textures/default_texture.png");
        let diffuse_texture = Texture::from_texture_data(
            device,
            queue,
            TextureData::from_bytes("default_texture", diffuse_bytes).unwrap(),
        )
        .unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sprite_renderer_texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
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

        let mut texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sprite_renderer_texture_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sprite_renderer_uniform_bind_group_layout"),
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
            label: Some("sprite_renderer_uniform_bind_group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite_renderer_render_pipeline_layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite_renderer_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &textured_vertex_shader_module,
                entry_point: "main",
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(FragmentState {
                module: &textured_fragment_shader_module,
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
            label: Some("sprite_renderer_vertex_buffer"),
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
            label: Some("sprite_renderer_instance_buffer"),
            size: INSTANCE_BUFFER_SIZE,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            uniform_bind_group,
            uniform_buffer,
            texture: diffuse_texture,
            texture_bind_group,
            texture_bind_group_layout,
            vertex_buffer,
            instance_buffer,
            instance_bind_groups: vec![],
            instances: vec![],
        }
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        sprite: &Sprite,
        transform_2d: &Transform2D,
        textures: &HashMap<String, Texture>,
    ) {
        let instance = Instance {
            model: (*transform_2d).into(),
            size: Vector2 {
                x: sprite.width,
                y: sprite.height,
            },
        };

        queue.write_buffer(
            &self.instance_buffer,
            self.instances.len() as u64 * std::mem::size_of::<InstanceRaw>() as u64,
            bytemuck::cast_slice(&[instance.to_raw()]),
        );

        let texture = textures.get(&sprite.texture).unwrap_or(&self.texture);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sprite_renderer_instance_bind_group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });
        self.instance_bind_groups.push(bind_group);
        self.instances.push(instance);
    }

    pub fn render<'rpass>(&'rpass mut self, render_pass: &mut RenderPass<'rpass>) {
        for (i, instance_bind_group) in self.instance_bind_groups.iter().enumerate() {
            let instance_index = i as u32;
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &instance_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw(0..6, instance_index..instance_index + 1);
        }

        self.instances.clear();
    }
}

struct Instance {
    model: cgmath::Matrix4<f32>,
    size: cgmath::Vector2<f32>,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: self.model.into(),
            size: [self.size.x, self.size.y],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
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
                    format: wgpu::VertexFormat::Float2,
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 7,
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
