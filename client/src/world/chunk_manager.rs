use bevy_ecs::prelude::*;
use opencuboids_common::{in_bounds, loop_3d_vec, BlockID, Chunk, CHUNK_SIZE};

use crate::render::{ChunkMesh, MainRenderer};

use super::{physics::WorldTransform, Player};

const RENDER_DISTANCE: i32 = 2;

#[derive(Default)]
pub struct ChunkManager {
    chunk_map: bevy_utils::HashMap<glam::IVec3, Chunk>,
    chunk_pos_center: Option<glam::IVec3>,
}

impl ChunkManager {
    pub fn get_block(&self, pos: glam::IVec3) -> BlockID {
        let chunk_pos = (pos.as_vec3() / CHUNK_SIZE as f32).floor().as_ivec3();
        self.chunk_map
            .get(&chunk_pos)
            .unwrap()
            .get_block(pos.as_uvec3() % CHUNK_SIZE as u32)
    }
}

pub fn chunk_update(
    mut chunk_manager: ResMut<ChunkManager>,
    renderer: Res<MainRenderer>,
    player_query: Query<(&WorldTransform, With<Player>)>,
    mut mesh_query: Query<(Entity, &mut WorldTransform, &mut ChunkMesh, Without<Player>)>,
    mut commands: Commands,
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
        // Get chunks 1 more chunk pos than renderered to handle chunk neighbours on the edges
        let start_pos = player_chunk_pos - RENDER_DISTANCE;
        let end_pos = player_chunk_pos + RENDER_DISTANCE;

        // Remove chunks not in the new bounds
        chunk_manager
            .chunk_map
            .drain_filter(|pos, _| in_bounds(*pos, player_chunk_pos, RENDER_DISTANCE));

        // Add new chunks
        loop_3d_vec!(start_pos, end_pos, |chunk_pos| {
            if !chunk_manager.chunk_map.contains_key(&chunk_pos) {
                let mut chunk = Chunk::new();
                gen_blocks(&mut chunk, chunk_pos);
                chunk_manager.chunk_map.insert(chunk_pos, chunk);
            }
        });

        // Loop around in a spiral adding chunks meshes
        for i in 0..RENDER_DISTANCE {
            let start_pos = player_chunk_pos - i;
            let end_pos = player_chunk_pos + i;
            loop_3d_vec!(start_pos, end_pos, |chunk_pos: glam::IVec3| {
                // Only create the mesh if it's outside the previous center pos
                if chunk_manager.chunk_pos_center.map_or(true, |center_pos| {
                    !in_bounds(chunk_pos, center_pos, RENDER_DISTANCE)
                }) {
                    let chunk = chunk_manager.chunk_map.get(&chunk_pos).unwrap();
                    let block_pos = chunk_pos.as_vec3() * CHUNK_SIZE as f32;
                    let mesh = ChunkMesh::new(&renderer.device, &chunk, chunk_pos, &chunk_manager);
                    if let Some(mesh) = mesh {
                        commands
                            .spawn()
                            .insert(WorldTransform {
                                position: block_pos,
                                ..Default::default()
                            })
                            .insert(mesh);
                    }
                }
            });
        }

        // Remove any chunk meshes outside render distance
        for (entity, _, mesh, _) in mesh_query.iter_mut() {
            if !in_bounds(mesh.chunk_pos, player_chunk_pos, RENDER_DISTANCE) {
                commands.entity(entity).despawn();
            }
        }

        chunk_manager.chunk_pos_center = Some(player_chunk_pos);
    }
}

fn gen_blocks(chunk: &mut Chunk, chunk_pos: glam::IVec3) {
    let start_pos = chunk_pos * CHUNK_SIZE as i32;
    let end_pos = start_pos + CHUNK_SIZE as i32;

    for x in start_pos.x..end_pos.x {
        for z in start_pos.z..end_pos.z {
            let trig_y = (f32::sin(x as f32 / 10.0) * f32::cos(z as f32 / 10.0) * 10.0) as i32;
            for y in start_pos.y..end_pos.y {
                if y < trig_y as i32 {
                    let local_pos = glam::ivec3(x, y, z).as_uvec3() % CHUNK_SIZE as u32;
                    chunk.set_block(local_pos, 1);
                }
            }
        }
    }
}
