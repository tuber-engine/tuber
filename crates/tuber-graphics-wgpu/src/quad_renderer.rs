use crate::texture::Texture;
use crate::Vertex;
use nalgebra::{Matrix4, Transform, Vector2, Vector3, Vector4};
use num_traits::identities::Zero;
use std::collections::HashMap;
use tuber_common::transform::Transform2D;
use tuber_graphics::camera::OrthographicCamera;
use tuber_graphics::low_level::QuadDescription;
use tuber_graphics::texture::TextureData;
use tuber_graphics::transform::IntoMatrix4;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroupLayout, BufferDescriptor, Device, FragmentState, Queue, RenderPass, RenderPipeline,
    ShaderModule, TextureFormat,
};

const MAX_INSTANCE_COUNT: u64 = 100_000;
const VERTEX_COUNT_PER_INSTANCE: u32 = 6;
const INSTANCE_BUFFER_SIZE: u64 = MAX_INSTANCE_COUNT * std::mem::size_of::<InstanceRaw>() as u64;

pub struct QuadInstanceMetadata {
    pub instance_bind_group: Option<wgpu::BindGroup>,
}

pub(crate) struct QuadRenderer {
    colored_pipeline: wgpu::RenderPipeline,
    textured_pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    _texture_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture: Texture,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instances_metadata: Vec<QuadInstanceMetadata>,
    instances: Vec<Instance>,
}

impl QuadRenderer {
    pub fn new(device: &Device, queue: &Queue, texture_format: &TextureFormat) -> Self {
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

        let textured_vertex_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/textured_shader.vert.spv"));
        let textured_fragment_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/textured_shader.frag.spv"));

        let default_texture_bytes = include_bytes!("./textures/default_texture.png");
        let default_texture = Texture::from_texture_data(
            device,
            queue,
            TextureData::from_bytes("default_texture", default_texture_bytes).unwrap(),
        )
        .unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("quad_renderer_texture_bind_group_layout"),
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

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("quad_renderer_texture_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&default_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&default_texture.sampler),
                },
            ],
        });

        let textured_pipeline = Self::create_textured_render_pipeline(
            device,
            &textured_vertex_shader_module,
            &textured_fragment_shader_module,
            texture_format,
            &uniform_bind_group_layout,
            &texture_bind_group_layout,
        );
        let colored_pipeline = Self::create_colored_quad_render_pipeline(
            &device,
            &uniform_bind_group_layout,
            texture_format,
        );

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("quad_renderer_vertex_buffer"),
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
            label: Some("quad_renderer_instance_buffer"),
            size: INSTANCE_BUFFER_SIZE,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            colored_pipeline,
            textured_pipeline,
            uniform_bind_group,
            uniform_buffer,
            texture: default_texture,
            _texture_bind_group: texture_bind_group,
            texture_bind_group_layout,
            vertex_buffer,
            instance_buffer,
            instances_metadata: vec![],
            instances: vec![],
        }
    }

    fn create_textured_render_pipeline(
        device: &Device,
        textured_vertex_shader_module: &ShaderModule,
        textured_fragment_shader_module: &ShaderModule,
        texture_format: &TextureFormat,
        uniform_bind_group_layout: &BindGroupLayout,
        texture_bind_group_layout: &BindGroupLayout,
    ) -> RenderPipeline {
        let textured_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("quad_renderer_textured_render_pipeline_layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad_renderer_textured_render_pipeline"),
            layout: Some(&textured_pipeline_layout),
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
        })
    }

    fn create_colored_quad_render_pipeline(
        device: &Device,
        uniform_bind_group_layout: &BindGroupLayout,
        texture_format: &TextureFormat,
    ) -> RenderPipeline {
        let colored_vertex_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/colored_shader.vert.spv"));
        let colored_fragment_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/colored_shader.frag.spv"));

        let colored_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("quad_renderer_colored_render_pipeline_layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad_renderer_colored_render_pipeline"),
            layout: Some(&colored_pipeline_layout),
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
        })
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        quad: &QuadDescription,
        transform_2d: &Transform2D,
        textures: &HashMap<String, Texture>,
    ) {
        if self.instances.len() == 0 {
            self.instances_metadata.clear();
        }

        let instance = Instance {
            model: (*transform_2d).into_matrix4(),
            color: Vector3::new(quad.color.0, quad.color.1, quad.color.2),
            size: Vector2::new(quad.width, quad.height),
            texture_rectangle: match &quad.texture {
                Some(texture_description) => texture_description.texture_region.into(),
                None => Vector4::zero(),
            },
        };

        queue.write_buffer(
            &self.instance_buffer,
            self.instances.len() as u64 * std::mem::size_of::<InstanceRaw>() as u64,
            bytemuck::cast_slice(&[instance.to_raw()]),
        );

        let instance_metadata = if let Some(texture_path) = &quad.texture {
            let texture = textures
                .get(&texture_path.identifier)
                .unwrap_or(&self.texture);
            QuadInstanceMetadata {
                instance_bind_group: Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("quad_renderer_textured_instance_bind_group"),
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
                })),
            }
        } else {
            QuadInstanceMetadata {
                instance_bind_group: None,
            }
        };

        self.instances_metadata.push(instance_metadata);
        self.instances.push(instance);
    }

    pub fn render<'rpass>(&'rpass mut self, render_pass: &mut RenderPass<'rpass>) {
        for (i, instance_metadata) in self.instances_metadata.iter().enumerate() {
            let instance_index = i as u32;

            if let Some(instance_bind_group) = &instance_metadata.instance_bind_group {
                render_pass.set_pipeline(&self.textured_pipeline);
                render_pass.set_bind_group(0, &instance_bind_group, &[]);
                render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            } else {
                render_pass.set_pipeline(&self.colored_pipeline);
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            }

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw(
                0..VERTEX_COUNT_PER_INSTANCE,
                instance_index..instance_index + 1,
            );
        }
        self.instances.clear();
    }

    pub fn set_camera(
        &mut self,
        queue: &Queue,
        camera: &OrthographicCamera,
        transform: &Transform2D,
    ) {
        let projection_matrix: Matrix4<f32> = Matrix4::new_orthographic(
            camera.left,
            camera.right,
            camera.bottom,
            camera.top,
            camera.near,
            camera.far,
        );
        let view_matrix: Matrix4<f32> = (*transform).into_matrix4();
        let view_proj = projection_matrix * view_matrix.try_inverse().unwrap();
        let uniform = Uniforms {
            view_proj: view_proj.into(),
        };
        queue.write_buffer(&self.uniform_buffer, 0u64, bytemuck::cast_slice(&[uniform]));
    }
}

struct Instance {
    model: Matrix4<f32>,
    color: Vector3<f32>,
    size: Vector2<f32>,
    texture_rectangle: Vector4<f32>,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: self.model.into(),
            color: [self.color.x, self.color.y, self.color.z],
            size: [self.size.x, self.size.y],
            texture_rectangle: [
                self.texture_rectangle.x,
                self.texture_rectangle.y,
                self.texture_rectangle.z,
                self.texture_rectangle.w,
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
    color: [f32; 3],
    size: [f32; 2],
    texture_rectangle: [f32; 4],
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
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float4,
                    offset: mem::size_of::<[f32; 21]>() as wgpu::BufferAddress,
                    shader_location: 9,
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
            view_proj: Matrix4::new_orthographic(0.0, 800.0, 600.0, 0.0, -100.0, 100.0).into(),
        }
    }
}
