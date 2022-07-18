use bevy_ecs::prelude::ResMut;
use winit::event::{DeviceEvent, KeyboardInput};

#[derive(Default)]
pub struct Input {
    key_map: bevy_utils::HashMap<winit::event::VirtualKeyCode, bool>,
    pub mouse_offset: glam::Vec2,
}

impl Input {
    pub fn is_key_pressed(&self, key_code: winit::event::VirtualKeyCode) -> bool {
        return *self.key_map.get(&key_code).unwrap_or(&false);
    }

    pub fn update(&mut self) {
        self.mouse_offset = glam::Vec2::ZERO;
    }

    pub fn update_system(mut res: ResMut<Self>) {
        res.update();
    }

    pub fn process_event(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::Key(KeyboardInput {
                virtual_keycode: Some(virtual_keycode),
                state,
                ..
            }) => {
                self.key_map.insert(
                    *virtual_keycode,
                    *state == winit::event::ElementState::Pressed,
                );
            }
            DeviceEvent::MouseMotion { delta } => {
                self.mouse_offset = glam::vec2(delta.0 as f32, delta.1 as f32);
            }
            _ => (),
        }
    }
}
