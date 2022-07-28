use bevy_ecs::prelude::*;
use opencuboids_common::{Chunk, CHUNK_SIZE, CHUNK_VOLUME, DIRECTION_TO_VECTOR};

use crate::world::{ChunkManager, WorldPosition};

use super::{
    bind_group::{BindGroup, BindGroupEntry},
    buffer::{new_buffer_quad_index, Buffer, DynamicBuffer},
    main_renderer::RenderInstance,
    render_pipeline::RenderPipeline,
    texture::Texture,
    MainRenderer,
};

// Worst case scenario of chunk: 3D chessboard pattern
const MAX_QUADS: usize = CHUNK_VOLUME / 2 * 6;

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: glam::Vec3,
    uvs: glam::Vec2,
    light_level: f32,
}

impl Vertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32],
    };
}

/// Uses a direction index then vertex index to index CUBE_VERTICES
const CUBE_INDICES: &[usize] = &[
    6, 7, 4, 5, // North
    3, 2, 1, 0, // South
    2, 6, 5, 1, // East
    7, 3, 0, 4, // West
    2, 3, 7, 6, // Top
    0, 1, 5, 4, // Bottom
];

const CUBE_VERTICES: &[glam::Vec3] = &[
    // Top face
    glam::vec3(0.0, 0.0, 0.0),
    glam::vec3(1.0, 0.0, 0.0),
    glam::vec3(1.0, 1.0, 0.0),
    glam::vec3(0.0, 1.0, 0.0),
    // Bottom face
    glam::vec3(0.0, 0.0, 1.0),
    glam::vec3(1.0, 0.0, 1.0),
    glam::vec3(1.0, 1.0, 1.0),
    glam::vec3(0.0, 1.0, 1.0),
];

const QUAD_UVS: &[glam::Vec2] = &[
    glam::vec2(0.0, 0.0),
    glam::vec2(1.0, 0.0),
    glam::vec2(1.0, 1.0),
    glam::vec2(0.0, 1.0),
];

/// Uses a direction index to index
const LIGHT_LEVELS: &[f32] = &[0.8, 0.8, 0.6, 0.6, 1.0, 0.4];

#[derive(Component)]
pub struct ChunkMesh {
    verticies: Vec<Vertex>,
    vertex_buffer: DynamicBuffer<Vertex>,
}

impl ChunkMesh {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            verticies: Vec::with_capacity(MAX_QUADS * 4),
            vertex_buffer: DynamicBuffer::new(
                device,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                MAX_QUADS * 4,
            ),
        }
    }

    pub fn regenerate(&mut self, queue: &wgpu::Queue, chunk: &Chunk, chunk_manager: &ChunkManager) {
        self.verticies.clear();
        let chunk_block_pos = chunk.pos * CHUNK_SIZE as i32;
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let block_pos = glam::uvec3(x as u32, y as u32, z as u32);
                    if chunk.get_block(block_pos) == 0 {
                        continue;
                    }

                    for dir_index in 0..6 {
                        let dir_vec = DIRECTION_TO_VECTOR[dir_index];
                        let neighbour_pos = block_pos.as_ivec3() + dir_vec + chunk_block_pos;
                        if chunk_manager.get_block(neighbour_pos) == 0 {
                            self.add_face(block_pos, dir_index);
                        }
                    }
                }
            }
        }

        self.vertex_buffer.update(&queue, &self.verticies);
    }

    fn add_face(&mut self, block_pos: glam::UVec3, direction: usize) {
        // Gen vertices
        for i in 0..4 {
            let vertex = CUBE_VERTICES[CUBE_INDICES[(direction * 4) + i]];
            self.verticies.push(Vertex {
                position: vertex + block_pos.as_vec3(),
                uvs: QUAD_UVS[i],
                light_level: LIGHT_LEVELS[direction],
            });
        }
    }
}

pub struct ChunkRenderer {
    index_buffer: Buffer<u16>,
    render_pipeline: RenderPipeline,
    texture_bind_group: BindGroup,
}

impl FromWorld for ChunkRenderer {
    fn from_world(world: &mut World) -> Self {
        let renderer = world.resource::<MainRenderer>();
        let device = &renderer.device;

        let diffuse_image = image::load_from_memory(include_bytes!("dirt.png")).unwrap();
        let diffuse_texture = Texture::new(device, &renderer.queue, &diffuse_image);

        let texture_bind_group = BindGroup::new(
            device,
            &[
                BindGroupEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                BindGroupEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        );

        let index_buffer = new_buffer_quad_index(&device, MAX_QUADS);

        let render_pipeline = RenderPipeline::new(
            &device,
            wgpu::include_wgsl!("chunk.wgsl"),
            &[
                &renderer.global_bind_group.layout,
                &texture_bind_group.layout,
            ],
            &[Vertex::LAYOUT],
            renderer.config.format,
            Some(renderer.depth_texture.format),
            &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..12,
            }],
        );

        Self {
            render_pipeline,
            index_buffer,
            texture_bind_group,
        }
    }
}

pub fn chunk_render(
    renderer: Res<MainRenderer>,
    mut render_instance: ResMut<Option<RenderInstance>>,
    chunk_renderer: Res<ChunkRenderer>,
    query: Query<(&WorldPosition, &ChunkMesh)>,
) {
    if let Some(instance) = render_instance.as_mut() {
        let mut render_pass = instance.begin_render_pass(Some(&renderer.depth_texture.view));

        render_pass.set_pipeline(&chunk_renderer.render_pipeline.pipeline);
        render_pass.set_bind_group(0, &renderer.global_bind_group.group, &[]);
        render_pass.set_bind_group(1, &chunk_renderer.texture_bind_group.group, &[]);

        render_pass.set_index_buffer(
            chunk_renderer.index_buffer.buf.slice(..),
            wgpu::IndexFormat::Uint16,
        );

        for (postition, mesh) in query.iter() {
            let index_count = mesh.verticies.len() / 4 * 6;
            if index_count > 0 {
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.buf.slice(..));
                render_pass.set_push_constants(
                    wgpu::ShaderStages::VERTEX,
                    0,
                    bytemuck::cast_slice(&[postition.0]),
                );

                render_pass.draw_indexed(0..index_count as u32, 0, 0..1);
            }
        }
    }
}
