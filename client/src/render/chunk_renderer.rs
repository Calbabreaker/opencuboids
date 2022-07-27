use bevy_ecs::prelude::*;
use opencuboids_common::{CHUNK_SIZE, CHUNK_VOLUME};

use crate::world::WorldPosition;

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
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: glam::Vec3,
    uvs: glam::Vec2,
}

impl Vertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2],
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

    pub fn regenerate(&mut self, queue: &wgpu::Queue) {
        self.verticies.clear();
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                // for z in 0..CHUNK_SIZE {
                let z = 0.0;
                let block_pos = glam::vec3(x as f32, y as f32, z as f32);
                for dir_index in 0..6 {
                    self.add_face(block_pos, dir_index);
                }
                // }
            }
        }

        self.vertex_buffer.update(&queue, &self.verticies);
    }

    fn add_face(&mut self, block_pos: glam::Vec3, direction: usize) {
        // Gen vertices
        for i in 0..4 {
            let vertex = CUBE_VERTICES[CUBE_INDICES[(direction * 4) + i]];
            self.verticies.push(Vertex {
                position: vertex + block_pos,
                uvs: QUAD_UVS[i],
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

        for (i, (postition, mesh)) in query.iter().enumerate() {
            renderer
                .position_buffer
                .update(&renderer.queue, &[postition.0.extend(0.0)]);

            render_pass.set_vertex_buffer(i as u32, mesh.vertex_buffer.buf.slice(..));

            let index_count = mesh.verticies.len() / 4 * 6;
            render_pass.draw_indexed(0..index_count as u32, 0, 0..1);
        }
    }
}
