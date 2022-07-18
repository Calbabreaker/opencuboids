use winit::event::{KeyboardInput, WindowEvent};

#[derive(Default)]
pub struct Input {
    key_map: bevy_utils::HashMap<winit::event::VirtualKeyCode, bool>,
    pub mouse_position: glam::Vec2,
    pub mouse_offset: glam::Vec2,
    last_mouse_position: Option<glam::Vec2>,
}

impl Input {
    pub fn is_key_pressed(&self, key_code: winit::event::VirtualKeyCode) -> bool {
        return *self.key_map.get(&key_code).unwrap_or(&false);
    }

    pub fn update(&mut self) {
        if let Some(last_mouse_position) = self.last_mouse_position {
            self.mouse_offset = self.mouse_position - last_mouse_position;
        }

        self.last_mouse_position = Some(self.mouse_position);
    }

    pub fn process_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(virtual_keycode),
                        state,
                        ..
                    },
                ..
            } => {
                self.key_map.insert(
                    *virtual_keycode,
                    *state == winit::event::ElementState::Pressed,
                );
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = glam::vec2(position.x as f32, position.y as f32);
            }
            _ => (),
        }
    }
}
