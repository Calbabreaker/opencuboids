use std::sync::Arc;

use bevy_ecs::prelude::*;
use wgpu::util::DeviceExt;

use super::{
    bind_group::{BindGroup, BindGroupEntry},
    render_pipeline::RenderPipeline,
    texture::Texture,
    MainRenderer,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    uvs: [f32; 2],
}

impl Vertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2],
    };
}

// note that y uvs are flipped to account for different y-axis
const QUAD_VERTICES: &[Vertex] = &[
    // bottom left
    Vertex {
        position: [-0.5, -0.5, 0.0],
        uvs: [0.0, 1.0],
    },
    // bottom right
    Vertex {
        position: [0.5, -0.5, 0.0],
        uvs: [1.0, 1.0],
    },
    // top right
    Vertex {
        position: [0.5, 0.5, 0.0],
        uvs: [1.0, 0.0],
    },
    // top left
    Vertex {
        position: [-0.5, 0.5, 0.0],
        uvs: [0.0, 0.0],
    },
];

const INDICES: &[u16] = &[
    0, 1, 2, // 1
    2, 3, 0, // 2
];

pub struct ChunkRenderer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    render_pipeline: RenderPipeline,
}

impl ChunkRenderer {
    pub fn new(renderer: &MainRenderer) -> Self {
        let device = &renderer.device;

        let diffuse_image = image::load_from_memory(include_bytes!("dirt.png")).unwrap();
        let diffuse_texture = Texture::new(renderer, &diffuse_image);

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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let render_pipeline = RenderPipeline::new(
            renderer,
            wgpu::include_wgsl!("chunk.wgsl"),
            &[
                renderer.global_bind_group.clone(),
                Arc::new(texture_bind_group),
            ],
            &[Vertex::LAYOUT],
        );

        Self {
            render_pipeline,
            index_buffer,
            vertex_buffer,
        }
    }
}

pub fn chunk_render_system(mut renderer: ResMut<MainRenderer>, chunk_renderer: Res<ChunkRenderer>) {
    if let Some(mut render_pass) = renderer.begin_render_pass(&chunk_renderer.render_pipeline) {
        render_pass.set_vertex_buffer(0, chunk_renderer.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            chunk_renderer.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
    }
}
