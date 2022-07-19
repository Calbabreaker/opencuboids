use crate::{camera::Camera, window::Window};
use bevy_ecs::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalUniform {
    pub view_projection: glam::Mat4,
}

pub struct Renderer {
    surface: wgpu::Surface,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub global_bind_group_layout: wgpu::BindGroupLayout,
    pub global_bind_group: wgpu::BindGroup,
    global_uniform_buffer: wgpu::Buffer,
    encoder: Option<wgpu::CommandEncoder>,
    view: Option<wgpu::TextureView>,
    output: Option<wgpu::SurfaceTexture>,
}

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window.handle) };
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
            size: std::mem::size_of::<GlobalUniform>() as wgpu::BufferAddress,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &global_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: global_uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            global_uniform_buffer,
            global_bind_group_layout,
            global_bind_group,
            encoder: None,
            view: None,
            output: None,
        }
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

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

    pub fn begin_render_pass(&mut self) -> Option<wgpu::RenderPass> {
        let mut render_pass =
            self.encoder
                .as_mut()?
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
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
        render_pass.set_bind_group(0, &self.global_bind_group, &[]);
        Some(render_pass)
    }
}

pub fn pre_render_system(
    mut renderer: ResMut<Renderer>,
    camera: Res<Camera>,
    mut viewport_resize_event: EventWriter<ViewportResizeEvent>,
) {
    let uniform = GlobalUniform {
        view_projection: camera.get_view_projection(),
    };

    renderer.queue.write_buffer(
        &renderer.global_uniform_buffer,
        0,
        bytemuck::cast_slice(&[uniform]),
    );

    match renderer.prepare_render() {
        Err(wgpu::SurfaceError::Lost) => {
            viewport_resize_event.send(ViewportResizeEvent(renderer.size));
        }
        Err(wgpu::SurfaceError::OutOfMemory) => panic!("GPU out of memory"),
        Err(e) => eprintln!("{:?}", e),
        Ok(_) => (),
    };
}

pub fn post_render_system(mut renderer: ResMut<Renderer>) {
    renderer.present_render();
}

pub struct ViewportResizeEvent(pub winit::dpi::PhysicalSize<u32>);

pub fn viewport_resize(
    mut renderer: ResMut<Renderer>,
    mut camera: ResMut<Camera>,
    mut viewport_resize_event: EventReader<ViewportResizeEvent>,
) {
    for event in viewport_resize_event.iter() {
        let size = event.0;
        renderer.resize(size);
        camera.resize(size.width, size.height);
    }
}
