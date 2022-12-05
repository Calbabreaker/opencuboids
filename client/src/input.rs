use bevy_ecs::prelude::*;
use std::hash::Hash;

use crate::window::{
    ElementState, KeyboardInput, MouseButton, MouseInput, MouseMotion, VirtualKeyCode,
};

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

impl<T: Copy + Eq + Hash> InputState<T> {
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

#[derive(Default, Resource)]
pub struct Input {
    key_state: InputState<VirtualKeyCode>,
    mouse_state: InputState<MouseButton>,
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
}

fn process_events(
    mut input: ResMut<Input>,
    mut keyboard_input_event: EventReader<KeyboardInput>,
    mut mouse_input_event: EventReader<MouseInput>,
    mut mouse_motion_event: EventReader<MouseMotion>,
) {
    input.mouse_offset = glam::Vec2::ZERO;
    input.key_state.clear();
    input.mouse_state.clear();

    for event in keyboard_input_event.iter() {
        match event.state {
            ElementState::Pressed => input.key_state.press(event.keycode),
            ElementState::Released => input.key_state.release(event.keycode),
        }
    }

    for event in mouse_input_event.iter() {
        match event.state {
            ElementState::Pressed => input.mouse_state.press(event.button),
            ElementState::Released => input.mouse_state.release(event.button),
        };
    }

    for event in mouse_motion_event.iter() {
        input.mouse_offset = event.delta;
    }
}

#[derive(Default)]
pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.init_resource::<Input>()
            .add_system_to_stage(bevy_app::CoreStage::PreUpdate, process_events);
    }
}
