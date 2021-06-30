use crate::texture::Texture;
use crate::Vertex;
use nalgebra::{Matrix4, Point3, Point4};
use std::collections::HashMap;
use tuber_common::tilemap::Tilemap;
use tuber_common::transform::{IntoMatrix4, Transform2D};
use tuber_graphics::camera::OrthographicCamera;
use tuber_graphics::texture::TextureAtlas;
use tuber_graphics::texture::TextureRegion;
use tuber_graphics::tilemap::TilemapRender;
use wgpu::util::DeviceExt;
use wgpu::{BufferDescriptor, Device, FragmentState, Queue, RenderPass, TextureFormat};

pub(crate) struct TilemapRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    tilemap_data: HashMap<String, TilemapRenderData>,
}

impl TilemapRenderer {
    pub fn new(device: &Device, texture_format: &TextureFormat) -> Self {
        let uniforms = Uniforms::new();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tilemap_renderer_uniform_buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("tilemap_renderer_uniform_bind_group_layout"),
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
            label: Some("tilemap_renderer_uniform_bind_group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let vertex_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/tilemap.vert.spv"));
        let fragment_shader_module =
            device.create_shader_module(&wgpu::include_spirv!("shaders/tilemap.frag.spv"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tilemap_renderer_texture_bind_group_layout"),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("tilemap_renderer_render_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout, &uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("tilemap_renderer_render_pipeline"),
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

        Self {
            pipeline,
            uniform_bind_group,
            uniform_buffer,
            bind_group_layout,
            tilemap_data: HashMap::new(),
        }
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        tilemap: &Tilemap,
        tilemap_render: &TilemapRender,
        texture_atlas: &TextureAtlas,
        transform: &Transform2D,
        textures: &HashMap<String, Texture>,
    ) {
        if !tilemap_render.dirty {
            return;
        }

        let buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: (tilemap.width * tilemap.height * 6 * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let texture_identifier = texture_atlas.texture_identifier();
        let texture = textures.get(texture_identifier).unwrap();
        let texture_width = texture.size.0 as f32;
        let texture_height = texture.size.1 as f32;

        for j in 0..tilemap.height {
            for i in 0..tilemap.width {
                let texture_region_identifier = if let Some(texture_region_identifier) =
                    (tilemap_render.tile_texture_function)(&tilemap.tiles[i + j * tilemap.width])
                {
                    texture_region_identifier
                } else {
                    continue;
                };
                let texture_region = texture_atlas
                    .texture_region(texture_region_identifier)
                    .unwrap();
                let texture_region = TextureRegion {
                    x: texture_region.x / texture_width,
                    y: texture_region.y / texture_height,
                    width: texture_region.width / texture_width,
                    height: texture_region.height / texture_height,
                };

                let transform_matrix = transform.into_matrix4();

                queue.write_buffer(
                    &buffer,
                    ((i + j * tilemap.width) * 6 * std::mem::size_of::<Vertex>()) as u64,
                    bytemuck::cast_slice(&[
                        Vertex {
                            position: (transform_matrix
                                * Point4::new(
                                    (i * tilemap.tile_width) as f32,
                                    (j * tilemap.tile_height) as f32,
                                    0.0,
                                    1.0,
                                ))
                            .xyz()
                            .into(),
                            color: [1.0, 1.0, 1.0],
                            tex_coords: [texture_region.x, texture_region.y],
                        },
                        Vertex {
                            position: (transform_matrix
                                * Point4::new(
                                    (i * tilemap.tile_width) as f32,
                                    (j * tilemap.tile_height + tilemap.tile_height) as f32,
                                    0.0,
                                    1.0,
                                ))
                            .xyz()
                            .into(),
                            color: [1.0, 1.0, 1.0],
                            tex_coords: [
                                texture_region.x,
                                texture_region.y + texture_region.height,
                            ],
                        },
                        Vertex {
                            position: (transform_matrix
                                * Point4::new(
                                    (i * tilemap.tile_width + tilemap.tile_width) as f32,
                                    (j * tilemap.tile_height) as f32,
                                    0.0,
                                    1.0,
                                ))
                            .xyz()
                            .into(),
                            color: [1.0, 1.0, 1.0],
                            tex_coords: [texture_region.x + texture_region.width, texture_region.y],
                        },
                        Vertex {
                            position: (transform_matrix
                                * Point4::new(
                                    (i * tilemap.tile_width + tilemap.tile_width) as f32,
                                    (j * tilemap.tile_height) as f32,
                                    0.0,
                                    1.0,
                                ))
                            .xyz()
                            .into(),
                            color: [1.0, 1.0, 1.0],
                            tex_coords: [texture_region.x + texture_region.width, texture_region.y],
                        },
                        Vertex {
                            position: (transform_matrix
                                * Point4::new(
                                    (i * tilemap.tile_width) as f32,
                                    (j * tilemap.tile_height + tilemap.tile_height) as f32,
                                    0.0,
                                    1.0,
                                ))
                            .xyz()
                            .into(),
                            color: [1.0, 1.0, 1.0],
                            tex_coords: [
                                texture_region.x,
                                texture_region.y + texture_region.height,
                            ],
                        },
                        Vertex {
                            position: (transform_matrix
                                * Point4::new(
                                    (i * tilemap.tile_width + tilemap.tile_width) as f32,
                                    (j * tilemap.tile_height + tilemap.tile_height) as f32,
                                    0.0,
                                    1.0,
                                ))
                            .xyz()
                            .into(),
                            color: [1.0, 1.0, 1.0],
                            tex_coords: [
                                texture_region.x + texture_region.width,
                                texture_region.y + texture_region.height,
                            ],
                        },
                    ]),
                )
            }
        }

        let bind_group = self.create_texture_bind_group(device, texture);

        self.tilemap_data.insert(
            tilemap_render.identifier.to_owned(),
            TilemapRenderData {
                vertex_data: buffer,
                vertex_count: tilemap.width * tilemap.height * 6,
                bind_group,
            },
        );
    }

    pub fn render<'rpass>(&'rpass mut self, render_pass: &mut RenderPass<'rpass>) {
        for tilemap_render_data in self.tilemap_data.values() {
            render_pass.set_pipeline(&self.pipeline);

            render_pass.set_bind_group(0, &tilemap_render_data.bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, tilemap_render_data.vertex_data.slice(..));
            render_pass.draw(0..tilemap_render_data.vertex_count as u32, 0..1);
        }
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

    fn create_texture_bind_group(&self, device: &Device, texture: &Texture) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tilemap_renderer_texture_bind_group"),
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

struct TilemapRenderData {
    vertex_data: wgpu::Buffer,
    vertex_count: usize,
    bind_group: wgpu::BindGroup,
}
