use std::collections::VecDeque;

use bevy_ecs::prelude::*;
use opencuboids_common::{in_bounds, loop_3d_vec, network, BlockID, Chunk, CHUNK_SIZE};

use crate::network::StreamChannel;

use super::{physics::WorldTransform, Player};

pub const RENDER_DISTANCE: i32 = 2;

#[derive(Default)]
pub struct ChunkManager {
    pub chunk_update_queue: VecDeque<glam::IVec3>,
    pub loading_chunks: bool,
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

    pub fn handle_chunk_response(&mut self, chunks: Vec<Chunk>) {
        for chunk in chunks {
            self.chunk_map.insert(chunk.pos, chunk);
        }
        self.loading_chunks = false;
    }
}

pub fn chunk_update(
    mut chunk_manager: ResMut<ChunkManager>,
    player_query: Query<(&WorldTransform, With<Player>)>,
    channel: ResMut<StreamChannel>,
) {
    let (player_trans, _) = player_query.single();

    // If the player has moved into a different chunk
    let player_chunk_pos = (player_trans.position / CHUNK_SIZE as f32)
        .floor()
        .as_ivec3();
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
        channel
            .sender
            .send(network::Request::ChunkRange { start, end })
            .ok();

        // Loop around in a spiral adding chunks meshes
        for i in 0..RENDER_DISTANCE {
            let start_pos = player_chunk_pos - i;
            let end_pos = player_chunk_pos + i;
            loop_3d_vec!(start_pos, end_pos, |chunk_pos: glam::IVec3| {
                // Only create the mesh if it's outside the previous center pos
                if chunk_manager.chunk_pos_center.map_or(true, |center_pos| {
                    !in_bounds(chunk_pos, center_pos, RENDER_DISTANCE)
                }) {
                    chunk_manager.chunk_update_queue.push_back(chunk_pos);
                }
            });
        }

        chunk_manager.loading_chunks = true;
        chunk_manager.chunk_pos_center = Some(player_chunk_pos);
    }
}
