use crate::{
    camera::Camera,
    window::{Window, WindowResize},
    world::WorldTransform,
};
use bevy_ecs::prelude::*;
use std::sync::Arc;

use super::{
    bind_group::{BindGroup, BindGroupEntry},
    buffer::DynamicBuffer,
    texture::Texture,
};

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalUniform {
    view_projection: glam::Mat4,
}

pub struct MainRenderer {
    surface: wgpu::Surface,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub global_bind_group: BindGroup,
    global_uniform_buffer: DynamicBuffer<GlobalUniform>,
    pub instance: Option<Arc<RenderInstance>>,
    pub depth_texture: Texture,
}

impl MainRenderer {
    pub async fn new(window: &Window) -> Self {
        // Set the WGPU_BACKEND env var as a comma seperated list of specific backend(s) to use
        let backends = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all());
        let instance = wgpu::Instance::new(backends);
        let surface = unsafe { instance.create_surface(&window.win) };
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
                    features: wgpu::Features::PUSH_CONSTANTS,
                    limits: wgpu::Limits {
                        max_push_constant_size: 12,
                        ..Default::default()
                    },
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
        let global_uniform_buffer = DynamicBuffer::new(&device, usage, 1);

        let global_bind_group = BindGroup::new(
            &device,
            &[BindGroupEntry::new_buffer(
                wgpu::ShaderStages::VERTEX,
                &global_uniform_buffer,
            )],
        );

        Self {
            depth_texture: Texture::new_depth(&device, &config),
            surface,
            device,
            queue,
            config,
            global_bind_group,
            global_uniform_buffer,
            instance: None,
        }
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth_texture = Texture::new_depth(&self.device, &self.config);
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
        depth_texture_view: Option<&'a wgpu::TextureView>,
    ) -> wgpu::RenderPass {
        self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            depth_stencil_attachment: depth_texture_view.map(|view| {
                wgpu::RenderPassDepthStencilAttachment {
                    view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }
            }),
        })
    }
}

pub fn pre_render(
    mut renderer: ResMut<MainRenderer>,
    mut render_instance: ResMut<Option<RenderInstance>>,
    window: Res<Window>,
    camera_query: Query<(&Camera, &WorldTransform)>,
) {
    let (camera, transform) = camera_query.single();
    renderer.global_uniform_buffer.update(
        &renderer.queue,
        &[GlobalUniform {
            view_projection: camera.view_projection(*transform),
        }],
    );

    match RenderInstance::new(&renderer.device, &renderer.surface) {
        Err(wgpu::SurfaceError::Lost) => renderer.resize(window.size()),
        Err(wgpu::SurfaceError::OutOfMemory) => panic!("GPU out of memory"),
        Err(e) => log::error!("{:?}", e),
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
    mut camera_query: Query<&mut Camera>,
    mut viewport_resize_event: EventReader<WindowResize>,
) {
    for event in viewport_resize_event.iter() {
        renderer.resize(event.size);
        for mut camera in camera_query.iter_mut() {
            camera.resize(event.size.width, event.size.height);
        }
    }
}
