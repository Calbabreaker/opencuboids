use bevy_ecs::prelude::*;
use winit::event::VirtualKeyCode;

use crate::{
    camera::Camera,
    input::Input,
    physics::{Position, Rotation, Velocity},
};

#[derive(Component)]
pub struct Player;

pub fn player_movement(
    input: Res<Input>,
    mut query: Query<(&mut Velocity, &mut Rotation, With<Player>)>,
) {
    let (mut velocity, mut rotation, _) = query.single_mut();

    const SENSITIVITY: f32 = 0.1;
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
    if input.is_key_pressed(VirtualKeyCode::Capital) {
        force -= glam::Vec3::Y;
    }

    velocity.vector = force.normalize_or_zero();
}

pub fn camera_update(
    mut camera: ResMut<Camera>,
    query: Query<(&Position, &Rotation, With<Player>)>,
) {
    let (position, rotation, _) = query.single();
    camera.update(position, rotation);
}
