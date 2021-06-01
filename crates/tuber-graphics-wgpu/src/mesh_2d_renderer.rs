use crate::texture::Texture;
use crate::Vertex;
use cgmath::{Matrix4, Point3, Transform};
use std::collections::HashMap;
use tuber_graphics::camera::OrthographicCamera;
use tuber_graphics::low_level::MeshDescription;
use tuber_graphics::texture::TextureData;
use tuber_graphics::Transform2D;
use wgpu::util::DeviceExt;
use wgpu::{BufferDescriptor, Device, FragmentState, Queue, RenderPass, TextureFormat};

// TODO remove and reallocate buffer dynamically
const MAX_VERTEX_COUNT: usize = 1000;

pub(crate) struct Mesh2DRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    texture_bind_groups: HashMap<String, wgpu::BindGroup>,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    texture: Texture,
    vertex_buffer: wgpu::Buffer,
    vertex_count: usize,
    mesh_metadata: Vec<MeshMetadata>,
}

impl Mesh2DRenderer {
    pub fn new(device: &Device, queue: &Queue, texture_format: &TextureFormat) -> Self {
        let uniforms = Uniforms::new();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh_2d_renderer_uniform_buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("mesh_2d_renderer_uniform_bind_group_layout"),
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
            label: Some("mesh_2d_renderer_uniform_bind_group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let vertex_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/mesh_2d_shader.vert.spv"));
        let fragment_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/mesh_2d_shader.frag.spv"));
        let default_texture_bytes = include_bytes!("./textures/default_texture.png");
        let default_texture = Texture::from_texture_data(
            device,
            queue,
            TextureData::from_bytes("default_texture", default_texture_bytes).unwrap(),
        )
        .unwrap();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mesh_2d_renderer_texture_bind_group_layout"),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mesh_2d_renderer_texture_bind_group"),
            layout: &bind_group_layout,
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh_2d_renderer_render_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout, &uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mesh_2d_renderer_render_pipeline"),
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
            label: Some("mesh_2d_renderer_vertex_buffer"),
            size: (MAX_VERTEX_COUNT * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            uniform_bind_group,
            uniform_buffer,
            bind_group,
            bind_group_layout,
            texture_bind_groups: HashMap::new(),
            texture: default_texture,
            vertex_buffer,
            vertex_count: 0,
            mesh_metadata: vec![],
        }
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        mesh_description: &MeshDescription,
        transform_2d: &Transform2D,
        textures: &HashMap<String, Texture>,
    ) {
        let transform_matrix: Matrix4<f32> = transform_2d.clone().into();
        for (vertex_index, vertex) in mesh_description.vertices.iter().enumerate() {
            let transformed_point = transform_matrix.transform_point(Point3::new(
                vertex.position.0,
                vertex.position.1,
                vertex.position.2,
            ));

            queue.write_buffer(
                &self.vertex_buffer,
                ((self.vertex_count + vertex_index) * std::mem::size_of::<Vertex>()) as u64,
                bytemuck::cast_slice(&[Vertex {
                    position: [
                        transformed_point.x,
                        transformed_point.y,
                        transformed_point.z,
                    ],
                    color: [vertex.color.0, vertex.color.1, vertex.color.2],
                    tex_coords: [vertex.texture_coordinates.0, vertex.texture_coordinates.1],
                }]),
            );
        }

        if !self
            .texture_bind_groups
            .contains_key(&mesh_description.texture.identifier)
        {
            self.texture_bind_groups.insert(
                mesh_description.texture.identifier.clone(),
                self.create_texture_bind_group(
                    &mesh_description.texture.identifier,
                    device,
                    textures,
                ),
            );
        }

        self.mesh_metadata.push(MeshMetadata {
            vertex_count: mesh_description.vertices.len(),
            texture_identifier: mesh_description.texture.identifier.to_owned(),
        });

        self.vertex_count += mesh_description.vertices.len();
    }

    pub fn render<'rpass>(&'rpass mut self, render_pass: &mut RenderPass<'rpass>) {
        let mut start_index = 0;
        for mesh_metadata in self.mesh_metadata.iter() {
            render_pass.set_pipeline(&self.pipeline);

            if let Some(bind_group) = self
                .texture_bind_groups
                .get(&mesh_metadata.texture_identifier)
            {
                render_pass.set_bind_group(0, bind_group, &[]);
            } else {
                render_pass.set_bind_group(0, &self.bind_group, &[]);
            }
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(
                start_index..start_index + mesh_metadata.vertex_count as u32,
                0..1,
            );

            start_index += mesh_metadata.vertex_count as u32;
        }

        self.mesh_metadata.clear();
        self.vertex_count = 0;
    }

    pub fn set_camera(
        &mut self,
        queue: &Queue,
        camera: &OrthographicCamera,
        transform: &Transform2D,
    ) {
        let projection_matrix: Matrix4<f32> = cgmath::ortho(
            camera.left,
            camera.right,
            camera.bottom,
            camera.top,
            camera.near,
            camera.far,
        );
        let view_matrix: Matrix4<f32> = (*transform).into();
        let view_proj = projection_matrix * view_matrix.inverse_transform().unwrap();
        let uniform = Uniforms {
            view_proj: view_proj.into(),
        };
        queue.write_buffer(&self.uniform_buffer, 0u64, bytemuck::cast_slice(&[uniform]));
    }

    fn create_texture_bind_group(
        &self,
        texture_identifier: &str,
        device: &Device,
        textures: &HashMap<String, Texture>,
    ) -> wgpu::BindGroup {
        let texture = textures.get(texture_identifier).unwrap();
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mesh_2d_renderer_texture_bind_group"),
            layout: &self.bind_group_layout,
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
        })
    }
}

struct MeshMetadata {
    vertex_count: usize,
    texture_identifier: String,
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
