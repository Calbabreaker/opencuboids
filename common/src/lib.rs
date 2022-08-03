mod chunk;
pub mod network;

pub use chunk::*;

pub const DEFAULT_PORT: u16 = 29707;

pub fn log_setup() {
    env_logger::Builder::default()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("opencuboids_client", log::LevelFilter::Trace)
        .filter_module("opencuboids_server", log::LevelFilter::Trace)
        .parse_default_env()
        .init();
}
