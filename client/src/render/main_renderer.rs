use crate::{
    camera::Camera,
    window::{Window, WindowResize},
};
use bevy_ecs::prelude::*;
use std::sync::Arc;

use super::{
    bind_group::{BindGroup, BindGroupEntry},
    buffer::DynamicBuffer,
    render_pipeline::RenderPipeline,
};

pub struct MainRenderer {
    surface: wgpu::Surface,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub global_bind_group: Arc<BindGroup>,
    pub position_buffer: DynamicBuffer<glam::Vec4>,
    view_projection_buffer: DynamicBuffer<glam::Mat4>,
    pub instance: Option<Arc<RenderInstance>>,
}

impl MainRenderer {
    pub async fn new(window: &Window) -> Self {
        // Set the WGPU_BACKEND env var as a comma seperated list of specific backend(s) to use
        let backends = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all());
        let instance = wgpu::Instance::new(backends);
        let surface = unsafe { instance.create_surface(&window.winit_window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find a GPU apdater!");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Failed to request a device!");

        let size = window.size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            present_mode: wgpu::PresentMode::Fifo,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
        };

        surface.configure(&device, &config);

        let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
        let view_projection_buffer = DynamicBuffer::new(&device, usage, 1);
        let position_buffer = DynamicBuffer::new(&device, usage, 1);

        let global_bind_group = BindGroup::new(
            &device,
            &[
                BindGroupEntry::new_buffer(wgpu::ShaderStages::VERTEX, &view_projection_buffer),
                BindGroupEntry::new_buffer(wgpu::ShaderStages::VERTEX, &position_buffer),
            ],
        );

        Self {
            surface,
            device,
            queue,
            config,
            global_bind_group: Arc::new(global_bind_group),
            view_projection_buffer,
            position_buffer,
            instance: None,
        }
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }
}

pub struct RenderInstance {
    encoder: wgpu::CommandEncoder,
    view: wgpu::TextureView,
    output: wgpu::SurfaceTexture,
}

impl RenderInstance {
    // Creates the output texture and encoder for rendering
    fn new(device: &wgpu::Device, surface: &wgpu::Surface) -> Result<Self, wgpu::SurfaceError> {
        let output = surface.get_current_texture()?;
        Ok(Self {
            view: output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
            encoder: device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None }),
            output,
        })
    }

    fn present(self, queue: &wgpu::Queue) {
        queue.submit(std::iter::once(self.encoder.finish()));
        self.output.present();
    }

    // Every render system should call this to start rendering
    pub fn begin_render_pass<'a>(
        &'a mut self,
        render_pipeline: &'a RenderPipeline,
    ) -> wgpu::RenderPass {
        let mut render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
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
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&render_pipeline.pipeline);
        for (i, bind_group) in render_pipeline.bind_groups.iter().enumerate() {
            render_pass.set_bind_group(i as u32, &bind_group.group, &[]);
        }

        render_pass
    }
}

pub fn pre_render(
    mut renderer: ResMut<MainRenderer>,
    mut render_instance: ResMut<Option<RenderInstance>>,
    window: Res<Window>,
    camera: Res<Camera>,
) {
    renderer
        .view_projection_buffer
        .update(&renderer.queue, &[camera.get_view_projection()]);

    match RenderInstance::new(&renderer.device, &renderer.surface) {
        Err(wgpu::SurfaceError::Lost) => renderer.resize(window.size()),
        Err(wgpu::SurfaceError::OutOfMemory) => panic!("GPU out of memory"),
        Err(e) => eprintln!("{:?}", e),
        Ok(instance) => *render_instance = Some(instance),
    };
}

pub fn post_render(
    renderer: Res<MainRenderer>,
    mut render_instance: ResMut<Option<RenderInstance>>,
) {
    if let Some(instance) = render_instance.take() {
        instance.present(&renderer.queue);
    }
}

pub fn on_resize(
    mut renderer: ResMut<MainRenderer>,
    mut camera: ResMut<Camera>,
    mut viewport_resize_event: EventReader<WindowResize>,
) {
    for event in viewport_resize_event.iter() {
        renderer.resize(event.size);
        camera.resize(event.size.width, event.size.height);
    }
}
