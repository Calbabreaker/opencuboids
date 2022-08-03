mod camera;
mod input;
mod network;
mod render;
mod time;
mod window;
mod world;

use bevy_app::App;
use opencuboids_common::DEFAULT_PORT;

fn main() {
    opencuboids_common::log_setup();
    let address = format!("0.0.0.0:{}", DEFAULT_PORT).parse().unwrap();
    std::thread::spawn(move || opencuboids_server::start(address));

    let channel = network::connect(address);

    App::new()
        .add_plugin(window::Plugin)
        .add_plugin(render::Plugin)
        .add_plugin(time::Plugin)
        .add_plugin(input::Plugin)
        .add_plugin(world::Plugin)
        .add_system(network::handle_responses)
        .insert_resource(channel)
        .run();
}
