use bevy_app::App;
use camera::Camera;
use opencuboids_common::DEFAULT_PORT;

mod camera;
mod input;
mod network;
mod render;
mod time;
mod window;
mod world;

fn main() {
    opencuboids_common::log_setup();
    std::thread::spawn(|| opencuboids_server::start(None));
    std::thread::spawn(|| network::connect(&"0.0.0.0".to_string(), DEFAULT_PORT));

    App::new()
        .add_plugin(window::WindowPlugin)
        .add_plugin(render::RenderPlugin)
        .add_plugin(time::TimePlugin)
        .add_plugin(input::InputPlugin)
        .add_plugin(world::WorldPlugin)
        .init_resource::<Camera>()
        .run();
}
