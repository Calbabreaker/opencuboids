use crate::state::State;
use bevy_ecs::prelude::*;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
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
        uv: [0.0, 1.0],
    },
    // bottom right
    Vertex {
        position: [0.5, -0.5, 0.0],
        uv: [1.0, 1.0],
    },
    // top right
    Vertex {
        position: [0.5, 0.5, 0.0],
        uv: [1.0, 0.0],
    },
    // top left
    Vertex {
        position: [-0.5, 0.5, 0.0],
        uv: [0.0, 0.0],
    },
];

const TRIANGLE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        uv: [0.5, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        uv: [1.0, 1.0],
    },
];

const INDICES: &[u16] = &[
    0, 1, 2, // 1
    2, 3, 0, // 2
];

pub struct RendererData {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl RendererData {
    pub fn new(state: &State) -> Self {
        let device = &state.device;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(TRIANGLE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(&wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::LAYOUT],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: state.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multiview: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        Self {
            render_pipeline,
            index_buffer,
            vertex_buffer,
        }
    }
}

fn render_wgpu(state: &State, renderer_data: &RendererData) -> Result<(), wgpu::SurfaceError> {
    let output = state.surface.get_current_texture()?;
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&renderer_data.render_pipeline);
        render_pass.set_vertex_buffer(0, renderer_data.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            renderer_data.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
    }

    state.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}

pub fn render(mut state: ResMut<State>, renderer_data: Res<RendererData>) {
    let size = state.size;
    match render_wgpu(&state, &renderer_data) {
        Err(wgpu::SurfaceError::Lost) => state.resize(size),
        Err(wgpu::SurfaceError::OutOfMemory) => panic!("GPU out of memory"),
        Err(e) => eprintln!("{:?}", e),
        Ok(_) => (),
    }
}
