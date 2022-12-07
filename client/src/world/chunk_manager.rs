use std::collections::VecDeque;

use bevy_ecs::prelude::*;
use noise::NoiseFn;
use opencuboids_common::{in_bounds, iter_3d_vec, BlockID, Chunk, CHUNK_SIZE};

use super::{physics::WorldTransform, Player};

pub const RENDER_DISTANCE: i32 = 5;

#[derive(Default, Resource)]
pub struct ChunkManager {
    pub chunk_update_queue: VecDeque<glam::IVec3>,
    pub chunks_left_loading: u32,
    pub chunk_map: bevy_utils::HashMap<glam::IVec3, Chunk>,
    pub chunk_pos_center: Option<glam::IVec3>,
}

impl ChunkManager {
    pub fn get_block(&self, pos: glam::IVec3) -> BlockID {
        let chunk_pos = (pos.as_vec3() / CHUNK_SIZE as f32).floor().as_ivec3();
        self.chunk_map
            .get(&chunk_pos)
            .unwrap()
            .get_block(pos.as_uvec3() % CHUNK_SIZE as u32)
    }

    pub fn handle_chunk_response(&mut self, chunk: Chunk) {
        self.chunk_map.insert(chunk.pos, chunk);
        log::info!("Loading {}", self.chunks_left_loading);
        self.chunks_left_loading -= 1;
    }
}

pub fn chunk_update(
    mut chunk_manager: ResMut<ChunkManager>,
    player_query: Query<(&WorldTransform, With<Player>)>,
) {
    let (player_trans, _) = player_query.single();

    // If the player has moved into a different chunk
    let player_chunk_pos = player_trans.position.as_ivec3() / CHUNK_SIZE as i32;
    if chunk_manager
        .chunk_pos_center
        .map_or(true, |pos| player_chunk_pos != pos)
    {
        // Remove chunks not in the new bounds
        chunk_manager
            .chunk_map
            .drain_filter(|pos, _| in_bounds(*pos, player_chunk_pos, RENDER_DISTANCE));

        // Get chunks 1 more chunk pos than renderered to handle chunk neighbours on the edges
        let start = player_chunk_pos - RENDER_DISTANCE;
        let end = player_chunk_pos + RENDER_DISTANCE;

        // Only client side for now because not working network
        for chunk_pos in iter_3d_vec(start, end) {
            if !chunk_manager.chunk_map.contains_key(&chunk_pos) {
                let mut chunk = Chunk::new(chunk_pos);
                gen_blocks(&mut chunk, chunk_pos);
                chunk_manager.chunk_map.insert(chunk.pos, chunk);
            }
        }

        // Loop around in a spiral adding chunks meshes
        for i in 0..RENDER_DISTANCE {
            let start_pos = player_chunk_pos - i;
            let end_pos = player_chunk_pos + i;
            for chunk_pos in iter_3d_vec(start_pos, end_pos) {
                // Only create the mesh if it's outside the previous center pos
                if chunk_manager.chunk_pos_center.map_or(true, |center_pos| {
                    !in_bounds(chunk_pos, center_pos, RENDER_DISTANCE)
                }) {
                    chunk_manager.chunk_update_queue.push_back(chunk_pos);
                }
            }
        }

        chunk_manager.chunk_pos_center = Some(player_chunk_pos);
    }
}

pub fn gen_blocks(chunk: &mut Chunk, chunk_pos: glam::IVec3) {
    let start_pos = chunk_pos * CHUNK_SIZE as i32;
    let end_pos = start_pos + CHUNK_SIZE as i32;
    let perlin = noise::Perlin::new(10000);

    for x in start_pos.x..end_pos.x {
        for z in start_pos.z..end_pos.z {
            let noise_y = perlin.get([x as f64 / 32.0, z as f64 / 32.0]) * 16.0;
            for y in start_pos.y..end_pos.y {
                if y < noise_y as i32 {
                    let local_pos = glam::ivec3(x, y, z).as_uvec3() % CHUNK_SIZE as u32;
                    chunk.set_block(local_pos, 1);
                }
            }
        }
    }
}
