use crate::{
    camera::Camera,
    window::{Window, WindowResize},
};
use bevy_ecs::prelude::*;
use std::sync::Arc;

use super::{
    bind_group::{BindGroup, BindGroupEntry},
    render_pipeline::RenderPipeline,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalUniform {
    pub view_projection: glam::Mat4,
}

pub struct MainRenderer {
    surface: wgpu::Surface,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub global_bind_group: Arc<BindGroup>,
    global_uniform_buffer: wgpu::Buffer,
    encoder: Option<wgpu::CommandEncoder>,
    view: Option<wgpu::TextureView>,
    output: Option<wgpu::SurfaceTexture>,
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

        let global_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<GlobalUniform>() as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let global_bind_group = BindGroup::new(
            &device,
            &[BindGroupEntry {
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                resource: global_uniform_buffer.as_entire_binding(),
            }],
        );

        Self {
            surface,
            device,
            queue,
            config,
            global_bind_group: Arc::new(global_bind_group),
            global_uniform_buffer,
            encoder: None,
            view: None,
            output: None,
        }
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    // Every render system should call this to start rendering
    pub fn begin_render_pass<'a>(
        &'a mut self,
        render_pipeline: &'a RenderPipeline,
    ) -> Option<wgpu::RenderPass> {
        let encoder = self.encoder.as_mut()?;
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.view.as_ref()?,
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

        Some(render_pass)
    }

    // Creates the output texture and encoder for rendering
    fn prepare_render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        self.view = Some(
            output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );
        self.encoder = Some(
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None }),
        );
        self.output = Some(output);
        Ok(())
    }

    fn present_render(&mut self) -> Option<()> {
        self.queue
            .submit(std::iter::once(self.encoder.take()?.finish()));
        self.output.take()?.present();
        self.view = None;
        Some(())
    }
}

pub fn pre_render(mut renderer: ResMut<MainRenderer>, window: Res<Window>, camera: Res<Camera>) {
    // Write the global uniform
    renderer.queue.write_buffer(
        &renderer.global_uniform_buffer,
        0,
        bytemuck::cast_slice(&[GlobalUniform {
            view_projection: camera.get_view_projection(),
        }]),
    );

    match renderer.prepare_render() {
        Err(wgpu::SurfaceError::Lost) => renderer.resize(window.size()),
        Err(wgpu::SurfaceError::OutOfMemory) => panic!("GPU out of memory"),
        Err(e) => eprintln!("{:?}", e),
        Ok(_) => (),
    };
}

pub fn post_render(mut renderer: ResMut<MainRenderer>) {
    renderer.present_render();
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
