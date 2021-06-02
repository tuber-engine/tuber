use crate::texture::Texture;
use crate::Vertex;
use cgmath::{Matrix4, Point3, Transform};
use std::collections::HashMap;
use tuber_common::tilemap::Tilemap;
use tuber_graphics::camera::OrthographicCamera;
use tuber_graphics::low_level::MeshDescription;
use tuber_graphics::texture::{TextureData, TextureRegion};
use tuber_graphics::tilemap::TilemapRender;
use tuber_graphics::{transform::Transform2D, TextureAtlas};
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupDescriptor, BufferDescriptor, Device, FragmentState, Queue, RenderPass, TextureFormat,
};

// TODO remove and reallocate buffer dynamically
const MAX_VERTEX_COUNT: usize = 1000;

pub(crate) struct TilemapRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    texture_bind_groups: HashMap<String, wgpu::BindGroup>,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    texture: Texture,
    vertex_buffer: wgpu::Buffer,
    vertex_count: usize,
    tilemap_data: HashMap<String, TilemapRenderData>,
}

impl TilemapRenderer {
    pub fn new(device: &Device, queue: &Queue, texture_format: &TextureFormat) -> Self {
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
        let default_texture_bytes = include_bytes!("./textures/default_texture.png");
        let default_texture = Texture::from_texture_data(
            device,
            queue,
            TextureData::from_bytes("default_texture", default_texture_bytes).unwrap(),
        )
        .unwrap();

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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tilemap_renderer_texture_bind_group"),
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

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("tilemap_renderer_vertex_buffer"),
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

        for j in 0..tilemap.height {
            for i in 0..tilemap.width {
                let texture =
                    (tilemap_render.tile_texture_function)(&tilemap.tiles[i + j * tilemap.width])
                        .unwrap();
                let texture_region = texture_atlas.texture_region(texture).unwrap();
                let texture_region = TextureRegion {
                    x: texture_region.x / 64.0,
                    y: texture_region.y / 64.0,
                    width: 16.0 / 64.0,
                    height: 16.0 / 64.0,
                };

                queue.write_buffer(
                    &buffer,
                    ((i + j * tilemap.width) * 6 * std::mem::size_of::<Vertex>()) as u64,
                    bytemuck::cast_slice(&[
                        Vertex {
                            position: [
                                (i * tilemap.tile_width) as f32,
                                (j * tilemap.tile_height) as f32,
                                0.0,
                            ],
                            color: [1.0, 1.0, 1.0],
                            tex_coords: [texture_region.x, texture_region.y],
                        },
                        Vertex {
                            position: [
                                (i * tilemap.tile_width) as f32,
                                (j * tilemap.tile_height + tilemap.tile_height) as f32,
                                0.0,
                            ],
                            color: [1.0, 1.0, 1.0],
                            tex_coords: [
                                texture_region.x,
                                texture_region.y + texture_region.height,
                            ],
                        },
                        Vertex {
                            position: [
                                (i * tilemap.tile_width + tilemap.tile_width) as f32,
                                (j * tilemap.tile_height) as f32,
                                0.0,
                            ],
                            color: [1.0, 1.0, 1.0],
                            tex_coords: [texture_region.x + texture_region.width, texture_region.y],
                        },
                        Vertex {
                            position: [
                                (i * tilemap.tile_width + tilemap.tile_width) as f32,
                                (j * tilemap.tile_height) as f32,
                                0.0,
                            ],
                            color: [1.0, 1.0, 1.0],
                            tex_coords: [texture_region.x + texture_region.width, texture_region.y],
                        },
                        Vertex {
                            position: [
                                (i * tilemap.tile_width) as f32,
                                (j * tilemap.tile_height + tilemap.tile_height) as f32,
                                0.0,
                            ],
                            color: [1.0, 1.0, 1.0],
                            tex_coords: [
                                texture_region.x,
                                texture_region.y + texture_region.height,
                            ],
                        },
                        Vertex {
                            position: [
                                (i * tilemap.tile_width + tilemap.tile_width) as f32,
                                (j * tilemap.tile_height + tilemap.tile_height) as f32,
                                0.0,
                            ],
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

        let texture_identifier = texture_atlas.texture_identifier();
        let texture = &textures[texture_identifier];
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
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
        });

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

struct TilemapRenderData {
    vertex_data: wgpu::Buffer,
    vertex_count: usize,
    bind_group: wgpu::BindGroup,
}
