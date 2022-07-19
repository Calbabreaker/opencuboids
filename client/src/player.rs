use bevy_ecs::prelude::*;
use winit::event::VirtualKeyCode;

use crate::{
    camera::Camera,
    input::Input,
    physics::{PhysicsBody, Position, Rotation},
    window::Window,
};

#[derive(Component)]
pub struct Player;

pub fn player_movement_system(
    input: Res<Input>,
    mut query: Query<(&mut PhysicsBody, &mut Rotation, With<Player>)>,
) {
    let (mut body, mut rotation, _) = query.single_mut();

    const SENSITIVITY: f32 = 0.2;
    rotation.x -= input.mouse_offset.x * SENSITIVITY;
    rotation.y = (rotation.y - input.mouse_offset.y * SENSITIVITY).clamp(-89.0, 89.0);

    let yaw = rotation.x.to_radians();
    let front = glam::vec3(yaw.cos(), 0.0, yaw.sin()).normalize();
    let right = front.cross(glam::Vec3::Y);

    let mut force = glam::Vec3::ZERO;
    if input.is_key_pressed(VirtualKeyCode::W) {
        force += front;
    }
    if input.is_key_pressed(VirtualKeyCode::S) {
        force -= front;
    }
    if input.is_key_pressed(VirtualKeyCode::D) {
        force -= right;
    }
    if input.is_key_pressed(VirtualKeyCode::A) {
        force += right;
    }

    if input.is_key_pressed(VirtualKeyCode::Space) {
        force += glam::Vec3::Y;
    }
    if input.is_key_pressed(VirtualKeyCode::LShift) {
        force -= glam::Vec3::Y;
    }

    body.acceleration = force.normalize_or_zero() * 2.0;
}

pub fn camera_update_system(
    mut camera: ResMut<Camera>,
    query: Query<(&Position, &Rotation, With<Player>)>,
) {
    let (position, rotation, _) = query.single();
    camera.update(position, rotation);
}

pub fn mouse_lock_system(mut state: ResMut<Window>, input: Res<Input>) {
    if input.is_key_just_pressed(VirtualKeyCode::Escape) {
        let locked = state.mouse_locked();
        state.set_mouse_lock(!locked);
    }
}
