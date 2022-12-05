use crate::{
    camera::Camera,
    window::{Window, WindowResize},
    world::WorldTransform,
};
use bevy_ecs::prelude::*;

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

#[derive(Resource)]
pub struct RenderState {
    surface: wgpu::Surface,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub global_bind_group: BindGroup,
    global_uniform_buffer: DynamicBuffer<GlobalUniform>,
    pub depth_texture: Texture,
}

impl RenderState {
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
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
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
        }
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth_texture = Texture::new_depth(&self.device, &self.config);
    }
}

struct RenderInstance {
    encoder: wgpu::CommandEncoder,
    view: wgpu::TextureView,
    output: wgpu::SurfaceTexture,
}

#[derive(Resource, Default)]
pub struct MainRenderer {
    instance: Option<RenderInstance>,
}

impl MainRenderer {
    // Creates the output texture and encoder for rendering
    fn begin(
        &mut self,
        device: &wgpu::Device,
        surface: &wgpu::Surface,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = surface.get_current_texture()?;
        self.instance = Some(RenderInstance {
            view: output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
            encoder: device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None }),
            output,
        });

        Ok(())
    }

    fn present(&mut self, queue: &wgpu::Queue) {
        let instance = self
            .instance
            .take()
            .expect("Tried to present before calling begin");
        queue.submit(std::iter::once(instance.encoder.finish()));
        instance.output.present();
    }

    // Every render system should call this to start rendering
    pub fn begin_render_pass<'a>(
        &'a mut self,
        depth_texture_view: Option<&'a wgpu::TextureView>,
    ) -> wgpu::RenderPass {
        let instance = self
            .instance
            .as_mut()
            .expect("Tried to begin render pass calling begin");
        instance
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &instance.view,
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
    mut render_state: ResMut<RenderState>,
    mut renderer: ResMut<MainRenderer>,
    window: Res<Window>,
    camera_query: Query<(&Camera, &WorldTransform)>,
) {
    let (camera, transform) = camera_query.single();
    render_state.global_uniform_buffer.update(
        &render_state.queue,
        &[GlobalUniform {
            view_projection: camera.view_projection(*transform),
        }],
    );

    match renderer.begin(&render_state.device, &render_state.surface) {
        Err(wgpu::SurfaceError::Lost) => render_state.resize(window.size()),
        Err(wgpu::SurfaceError::OutOfMemory) => panic!("GPU out of memory"),
        Err(e) => log::error!("{:?}", e),
        _ => (),
    };
}

pub fn post_render(render_state: ResMut<RenderState>, mut renderer: ResMut<MainRenderer>) {
    renderer.present(&render_state.queue);
}

pub fn on_resize(
    mut renderer: ResMut<RenderState>,
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
