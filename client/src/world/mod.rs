mod chunk_manager;
mod physics;
mod player;

use self::{
    chunk_manager::chunk_update,
    physics::physics,
    player::{camera_update, mouse_lock, player_movement, Player},
};
use bevy_ecs::prelude::*;

pub use self::{
    chunk_manager::ChunkManager,
    physics::{PhysicsBody, WorldPosition, WorldRotation},
};

fn spawn(mut commands: Commands) {
    commands
        .spawn()
        .insert(WorldPosition(glam::vec3(0.0, 0.0, -2.0)))
        .insert(PhysicsBody::default())
        .insert(WorldRotation { x: 90.0, y: 0.0 })
        .insert(Player);
}

#[derive(Default)]
pub struct WorldPlugin;

impl bevy_app::Plugin for WorldPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.init_resource::<ChunkManager>()
            .add_startup_system(spawn)
            .add_system(chunk_update)
            .add_system(player_movement.before(physics))
            .add_system(physics)
            .add_system(mouse_lock)
            .add_system(camera_update.after(physics));
    }
}
