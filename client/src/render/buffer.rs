use wgpu::util::DeviceExt;

pub struct Buffer<T> {
    pub buf: wgpu::Buffer,
    _marker: std::marker::PhantomData<T>,
}

impl<T: bytemuck::Pod> Buffer<T> {
    pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, data: &[T]) -> Self {
        Self {
            buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(data),
                usage,
            }),
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates index buffer that indexes to make a quad (2 triangles)
    pub fn new_quad_index(device: &wgpu::Device, quad_count: usize) -> Buffer<u16> {
        let indicies = [0, 1, 2, 2, 3, 0]
            .iter()
            .cycle()
            .take(quad_count * 6)
            .enumerate()
            .map(|(i, quad_i)| (i / 6 * 4 + quad_i) as u16)
            .collect::<Vec<_>>();
        Buffer::new(device, wgpu::BufferUsages::INDEX, indicies.as_slice())
    }
}

pub struct DynamicBuffer<T> {
    pub buf: wgpu::Buffer,
    _marker: std::marker::PhantomData<T>,
}

impl<T: bytemuck::Pod> DynamicBuffer<T> {
    pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, len: usize) -> Self {
        Self {
            buf: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                mapped_at_creation: false,
                size: (len * std::mem::size_of::<T>()) as u64,
                usage,
            }),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, data: &[T]) {
        queue.write_buffer(&self.buf, 0, bytemuck::cast_slice(data));
    }
}
