pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = 32 * 32 * 32;

/// Uses a direction index
pub const DIRECTION_TO_VECTOR: &[glam::IVec3] = &[
    glam::ivec3(0, 0, 1),  // North
    glam::ivec3(0, 0, -1), // South
    glam::ivec3(1, 0, 0),  // East
    glam::ivec3(-1, 0, 0), // West
    glam::ivec3(0, 1, 0),  // Top
    glam::ivec3(0, -1, 0), // Bottom
];

pub const DEFAULT_PORT: u16 = 29707;

pub fn log_setup() {
    env_logger::Builder::default()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("opencuboids_client", log::LevelFilter::Trace)
        .filter_module("opencuboids_server", log::LevelFilter::Trace)
        .parse_default_env()
        .init();
}
