use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowBuilder};

pub struct Window {
    pub winit_window: winit::window::Window,
    mouse_locked: bool,
}

impl Window {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        Self {
            winit_window: WindowBuilder::new()
                .with_title("Opencuboids")
                .build(event_loop)
                .unwrap(),
            mouse_locked: false,
        }
    }

    pub fn set_mouse_lock(&mut self, locked: bool) {
        self.winit_window
            .set_cursor_grab(locked)
            .expect("Failed to lock mouse!");
        self.winit_window.set_cursor_visible(!locked);
        self.mouse_locked = locked;
    }

    pub fn mouse_locked(&self) -> bool {
        self.mouse_locked
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.winit_window.inner_size()
    }
}
