use bevy_ecs::prelude::ResMut;
use std::hash::Hash;
use winit::event::{ButtonId, DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode};

struct InputState<T: Eq + Hash> {
    pressed: bevy_utils::HashSet<T>,
    just_pressed: bevy_utils::HashSet<T>,
    just_released: bevy_utils::HashSet<T>,
}

impl<T: Eq + Hash> Default for InputState<T> {
    fn default() -> Self {
        Self {
            pressed: Default::default(),
            just_pressed: Default::default(),
            just_released: Default::default(),
        }
    }
}

impl<T: Copy + Eq + Hash + std::fmt::Debug> InputState<T> {
    fn press(&mut self, code: T) {
        self.pressed.insert(code);
        self.just_pressed.insert(code);
    }

    fn release(&mut self, code: T) {
        self.pressed.remove(&code);
        self.just_released.insert(code);
    }

    fn clear(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}

#[derive(Default)]
pub struct Input {
    key_state: InputState<VirtualKeyCode>,
    mouse_state: InputState<ButtonId>,
    pub mouse_offset: glam::Vec2,
}

impl Input {
    pub fn is_key_pressed(&self, key_code: VirtualKeyCode) -> bool {
        self.key_state.pressed.contains(&key_code)
    }

    pub fn is_key_just_pressed(&self, key_code: VirtualKeyCode) -> bool {
        self.key_state.just_pressed.contains(&key_code)
    }

    // pub fn is_key_just_released(&self, key_code: VirtualKeyCode) -> bool {
    //     self.key_state.just_released.contains(&key_code)
    // }

    // pub fn is_mouse_pressed(&self, mouse_code: ButtonId) -> bool {
    //     self.mouse_state.pressed.contains(&mouse_code)
    // }

    // pub fn is_mouse_just_pressed(&self, mouse_code: ButtonId) -> bool {
    //     self.mouse_state.just_pressed.contains(&mouse_code)
    // }

    // pub fn is_mouse_just_released(&self, mouse_code: ButtonId) -> bool {
    //     self.mouse_state.just_released.contains(&mouse_code)
    // }

    pub fn update(&mut self) {
        self.mouse_offset = glam::Vec2::ZERO;
        self.key_state.clear();
        self.mouse_state.clear();
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
            }) => match state {
                ElementState::Pressed => self.key_state.press(*virtual_keycode),
                ElementState::Released => self.key_state.release(*virtual_keycode),
            },
            DeviceEvent::Button { button, state } => match state {
                ElementState::Pressed => self.mouse_state.press(*button),
                ElementState::Released => self.mouse_state.release(*button),
            },
            DeviceEvent::MouseMotion { delta } => {
                self.mouse_offset = glam::vec2(delta.0 as f32, delta.1 as f32);
            }
            _ => (),
        }
    }
}
