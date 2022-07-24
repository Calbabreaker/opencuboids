use super::buffer::DynamicBuffer;

pub struct BindGroupEntry<'a> {
    pub ty: wgpu::BindingType,
    pub resource: wgpu::BindingResource<'a>,
    pub visibility: wgpu::ShaderStages,
}

impl<'a> BindGroupEntry<'a> {
    pub fn new_buffer<T>(visibility: wgpu::ShaderStages, buffer: &'a DynamicBuffer<T>) -> Self {
        Self {
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            visibility,
            resource: buffer.buf.as_entire_binding(),
        }
    }
}

pub struct BindGroup {
    pub group: wgpu::BindGroup,
    pub layout: wgpu::BindGroupLayout,
}

impl BindGroup {
    pub fn new(device: &wgpu::Device, entries: &[BindGroupEntry]) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &entries
                .iter()
                .enumerate()
                .map(|(i, entry)| wgpu::BindGroupLayoutEntry {
                    binding: i as u32,
                    visibility: entry.visibility,
                    ty: entry.ty,
                    count: None,
                })
                .collect::<Vec<_>>(),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &entries
                .iter()
                .enumerate()
                .map(|(i, entry)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: entry.resource.clone(),
                })
                .collect::<Vec<_>>(),
        });

        Self {
            layout: bind_group_layout,
            group: bind_group,
        }
    }
}
