use super::{PhysicsBody, WorldTransform};
use crate::{input::Input, window::Window};
use bevy_ecs::prelude::*;
use winit::event::VirtualKeyCode;

#[derive(Component)]
pub struct Player;

pub fn player_movement(
    input: Res<Input>,
    mut query: Query<(&mut PhysicsBody, &mut WorldTransform, With<Player>)>,
) {
    let (mut body, mut transform, _) = query.single_mut();

    const SENSITIVITY: f32 = 0.1;
    let mut rotation = &mut transform.rotation;
    rotation.x -= input.mouse_offset.x * SENSITIVITY;
    rotation.y = f32::clamp(rotation.y - input.mouse_offset.y * SENSITIVITY, -89.0, 89.0);

    // Only in the xz plane
    let yaw = rotation.x.to_radians();
    let front = glam::vec3(yaw.cos(), 0.0, yaw.sin());
    let left = front.cross(glam::Vec3::Y);
    let mut force = glam::Vec3::ZERO;

    // xz plane movement
    if input.is_key_pressed(VirtualKeyCode::W) {
        force += front;
    }
    if input.is_key_pressed(VirtualKeyCode::S) {
        force -= front;
    }
    if input.is_key_pressed(VirtualKeyCode::A) {
        force += left;
    }
    if input.is_key_pressed(VirtualKeyCode::D) {
        force -= left;
    }

    // y movement
    if input.is_key_pressed(VirtualKeyCode::Space) {
        force += glam::Vec3::Y;
    }
    if input.is_key_pressed(VirtualKeyCode::LShift) {
        force -= glam::Vec3::Y;
    }

    const SPEED: f32 = 10.0;
    body.force = force.normalize_or_zero() * SPEED;
}

pub fn mouse_lock(mut state: ResMut<Window>, input: Res<Input>) {
    if input.is_key_just_pressed(VirtualKeyCode::Escape) {
        let locked = state.mouse_locked();
        state.set_mouse_lock(!locked);
    }
}
