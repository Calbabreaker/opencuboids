mod chunk_manager;
mod physics;
mod player;

use crate::camera::Camera;
use bevy_ecs::prelude::*;

use self::{
    chunk_manager::chunk_update,
    physics::physics,
    player::{mouse_lock, player_movement, Player},
};
pub use self::{
    chunk_manager::{ChunkManager, RENDER_DISTANCE},
    physics::{PhysicsBody, WorldTransform},
};

fn spawn(mut commands: Commands) {
    commands
        .spawn()
        .insert(WorldTransform {
            rotation: glam::vec2(f32::to_radians(90.0), 0.0),
            ..Default::default()
        })
        .insert(PhysicsBody::default())
        .insert(Camera::default())
        .insert(Player);
}

#[derive(Default)]
pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.init_resource::<ChunkManager>()
            .add_startup_system(spawn)
            .add_system(chunk_update)
            .add_system(player_movement.before(physics))
            .add_system(physics)
            .add_system(mouse_lock);
    }
}
