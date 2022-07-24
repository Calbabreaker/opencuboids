use super::{bind_group::BindGroup, MainRenderer};
use std::sync::Arc;

pub struct RenderPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_groups: Vec<Arc<BindGroup>>,
}

impl RenderPipeline {
    pub fn new(
        renderer: &MainRenderer,
        shader: wgpu::ShaderModuleDescriptor,
        bind_groups: &[Arc<BindGroup>],
        vertex_buffer_layouts: &[wgpu::VertexBufferLayout],
    ) -> Self {
        let device = &renderer.device;

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &bind_groups
                    .iter()
                    .map(|bind_group| &bind_group.layout)
                    .collect::<Vec<_>>(),
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(shader);
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: vertex_buffer_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: renderer.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
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
            pipeline: render_pipeline,
            bind_groups: bind_groups.to_vec(),
        }
    }
}
