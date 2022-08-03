use bevy_ecs::event::Events;
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub use winit::event::{ElementState, MouseButton, VirtualKeyCode};

pub struct WindowResize {
    pub size: winit::dpi::PhysicalSize<u32>,
}

pub struct MouseMotion {
    pub delta: glam::Vec2,
}

pub struct MouseInput {
    pub state: ElementState,
    pub button: MouseButton,
}

pub struct KeyboardInput {
    pub state: ElementState,
    pub keycode: VirtualKeyCode,
}

pub struct Window {
    pub win: winit::window::Window,
    mouse_locked: bool,
}

impl Window {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        Self {
            win: WindowBuilder::new()
                .with_title("Opencuboids")
                .build(event_loop)
                .unwrap(),
            mouse_locked: false,
        }
    }

    pub fn set_mouse_lock(&mut self, locked: bool) {
        self.win
            .set_cursor_grab(locked)
            .expect("Failed to lock mouse!");
        self.win.set_cursor_visible(!locked);
        self.mouse_locked = locked;
    }

    pub fn mouse_locked(&self) -> bool {
        self.mouse_locked
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.win.inner_size()
    }
}

fn runner(mut app: bevy_app::App) {
    let event_loop = app
        .world
        .remove_non_send_resource::<EventLoop<()>>()
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        let world = &mut app.world;

        match event {
            Event::MainEventsCleared => app.update(),
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    let mut events = world.resource_mut::<Events<WindowResize>>();
                    events.send(WindowResize { size: *size });
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    let mut events = world.resource_mut::<Events<WindowResize>>();
                    events.send(WindowResize {
                        size: **new_inner_size,
                    });
                }
                WindowEvent::KeyboardInput {
                    input:
                        winit::event::KeyboardInput {
                            state,
                            virtual_keycode: Some(keycode),
                            ..
                        },
                    ..
                } => {
                    let mut events = world.resource_mut::<Events<KeyboardInput>>();
                    events.send(KeyboardInput {
                        state: *state,
                        keycode: *keycode,
                    });
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    let mut events = world.resource_mut::<Events<MouseInput>>();
                    events.send(MouseInput {
                        state: *state,
                        button: *button,
                    });
                }
                _ => (),
            },
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta: (x, y) },
                ..
            } => {
                let mut events = world.resource_mut::<Events<MouseMotion>>();
                events.send(MouseMotion {
                    delta: glam::vec2(x as f32, y as f32),
                });
            }
            _ => (),
        }
    });
}

#[derive(Default)]
pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        let event_loop = EventLoop::new();
        app.insert_resource(Window::new(&event_loop))
            .insert_non_send_resource(event_loop)
            .add_event::<WindowResize>()
            .add_event::<MouseMotion>()
            .add_event::<MouseInput>()
            .add_event::<KeyboardInput>()
            .set_runner(runner);
    }
}
