use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowBuilder};

pub struct Window {
    pub handle: winit::window::Window,
    mouse_locked: bool,
}

impl Window {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        Self {
            handle: WindowBuilder::new()
                .with_title("Opencuboids")
                .build(event_loop)
                .unwrap(),
            mouse_locked: false,
        }
    }

    pub fn set_mouse_lock(&mut self, locked: bool) {
        self.handle
            .set_cursor_grab(locked)
            .expect("Failed to lock mouse!");
        self.handle.set_cursor_visible(!locked);
        self.mouse_locked = locked;
    }

    pub fn mouse_locked(&self) -> bool {
        self.mouse_locked
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.handle.inner_size()
    }
}
