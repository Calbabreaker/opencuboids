use bevy_app::App;
use bevy_ecs::prelude::*;
use camera::Camera;
use opencuboids_common::DEFAULT_PORT;

mod camera;
mod input;
mod network;
mod render;
mod time;
mod window;
mod world;

fn setup(mut commands: Commands, renderer: Res<render::MainRenderer>) {
    commands
        .spawn()
        .insert(world::WorldPosition(glam::vec3(0.0, 0.0, -2.0)))
        .insert(world::PhysicsBody::default())
        .insert(world::WorldRotation { x: 90.0, y: 0.0 })
        .insert(world::Player);

    let mut mesh = render::ChunkMesh::new(&renderer.device);
    mesh.regenerate(&renderer.queue);
    commands
        .spawn()
        .insert(world::WorldPosition::default())
        .insert(mesh);
}

fn main() {
    opencuboids_common::log_setup();
    std::thread::spawn(|| opencuboids_server::start(None));
    std::thread::spawn(|| network::connect(&"0.0.0.0".to_string(), DEFAULT_PORT));

    App::new()
        .add_plugin(window::WindowPlugin)
        .add_plugin(render::RenderPlugin)
        .add_plugin(time::TimePlugin)
        .add_plugin(input::InputPlugin)
        .init_resource::<Camera>()
        .add_startup_system(setup)
        .add_system(world::player_movement.before(world::physics))
        .add_system(world::physics)
        .add_system(world::mouse_lock)
        .add_system(world::camera_update.after(world::physics))
        .run();
}
