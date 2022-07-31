use bevy_ecs::prelude::*;
use opencuboids_common::{loop_3d, Chunk, CHUNK_SIZE, CHUNK_VOLUME, DIRECTION_TO_VECTOR};

use crate::world::{ChunkManager, WorldTransform};

use super::{
    bind_group::{BindGroup, BindGroupEntry},
    buffer::{new_buffer_quad_index, Buffer},
    main_renderer::RenderInstance,
    render_pipeline::RenderPipeline,
    texture::Texture,
    MainRenderer,
};

// Worst case scenario of chunk: 3D chessboard pattern
const MAX_QUADS: usize = CHUNK_VOLUME / 2 * 6;

/// Vertex is a packed 32-bit unsigned int containing all the vertex data
///    x      y      z    uv dir
/// |‾‾‾‾‾||‾‾‾‾‾| |‾‾‾‾‾||‾||‾|
/// 0000 0000 0000 0000 0000 0000 0000 0000  
#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex(u32);

impl Vertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Uint32],
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

const CUBE_VERTICES: &[glam::UVec3] = &[
    // Top face
    glam::uvec3(0, 0, 0),
    glam::uvec3(1, 0, 0),
    glam::uvec3(1, 1, 0),
    glam::uvec3(0, 1, 0),
    // Bottom face
    glam::uvec3(0, 0, 1),
    glam::uvec3(1, 0, 1),
    glam::uvec3(1, 1, 1),
    glam::uvec3(0, 1, 1),
];

#[derive(Component)]
pub struct ChunkMesh {
    vertex_buffer: Buffer<Vertex>,
    pub chunk_pos: glam::IVec3,
}

impl ChunkMesh {
    pub fn new(
        device: &wgpu::Device,
        chunk: &Chunk,
        chunk_pos: glam::IVec3,
        chunk_manager: &ChunkManager,
    ) -> Option<Self> {
        let mut verticies = [Vertex::default(); MAX_QUADS * 4];
        let mut vertex_i = 0;

        let chunk_block_pos = chunk_pos * CHUNK_SIZE as i32;
        loop_3d!(0..CHUNK_SIZE as i32, |block_pos: glam::IVec3| {
            if chunk.get_block(block_pos.as_uvec3()) == 0 {
                return;
            }

            for dir_index in 0..6 {
                let dir_vec = DIRECTION_TO_VECTOR[dir_index];
                let neighbour_pos = block_pos + dir_vec + chunk_block_pos;
                if chunk_manager.get_block(neighbour_pos) == 0 {
                    // Add face
                    for i in 0..4 {
                        let pos =
                            CUBE_VERTICES[CUBE_INDICES[(dir_index * 4) + i]] + block_pos.as_uvec3();

                        // Pack vertex data
                        let vertex = pos.x
                            | pos.y << 6
                            | pos.z << 12
                            | (i as u32) << 18
                            | (dir_index as u32) << 20;
                        verticies[vertex_i] = Vertex(vertex);
                        vertex_i += 1;
                    }
                }
            }
        });

        if vertex_i > 0 {
            Some(Self {
                vertex_buffer: Buffer::new(
                    &device,
                    wgpu::BufferUsages::VERTEX,
                    bytemuck::cast_slice(&verticies[0..vertex_i + 1]),
                ),
                chunk_pos,
            })
        } else {
            None
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
    query: Query<(&WorldTransform, &ChunkMesh)>,
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

        for (transform, mesh) in query.iter() {
            let index_count = mesh.vertex_buffer.len / 4 * 6;
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.buf.slice(..));
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                bytemuck::cast_slice(&[transform.position]),
            );

            render_pass.draw_indexed(0..index_count as u32, 0, 0..1);
        }
    }
}
