use bevy_ecs::prelude::*;
use opencuboids_common::{BlockID, Chunk, CHUNK_SIZE};

use crate::render::{ChunkMesh, MainRenderer};

use super::{Player, WorldPosition};

#[derive(Default)]
pub struct ChunkManager {
    chunk_map: bevy_utils::HashMap<glam::IVec3, Chunk>,
    chunk_pos_center: Option<glam::IVec3>,
}

impl ChunkManager {
    pub fn get_block(&self, pos: glam::IVec3) -> BlockID {
        let chunk_pos = (pos.as_vec3() / CHUNK_SIZE as f32).floor().as_ivec3();
        if let Some(chunk) = self.chunk_map.get(&chunk_pos) {
            chunk.get_block(pos.as_uvec3() % CHUNK_SIZE as u32)
        } else {
            0
        }
    }
}

pub fn chunk_update(
    mut chunk_manager: ResMut<ChunkManager>,
    renderer: Res<MainRenderer>,
    player_query: Query<(&WorldPosition, With<Player>)>,
    mut mesh_query: Query<(Entity, &mut WorldPosition, &mut ChunkMesh, Without<Player>)>,
    mut commands: Commands,
) {
    let (player_pos, _) = player_query.single();

    // If the player has moved into a different chunk
    let player_chunk_pos = (player_pos.0 / CHUNK_SIZE as f32).floor().as_ivec3();
    if chunk_manager
        .chunk_pos_center
        .map_or(true, |pos| player_chunk_pos != pos)
    {
        chunk_manager.chunk_pos_center = Some(player_chunk_pos);
        chunk_manager.chunk_map.clear();

        // Update chunks
        let mut mesh_iter = mesh_query.iter_mut();
        const RENDER_DISTANCE: usize = 3;
        for i in 0..RENDER_DISTANCE {
            let start_pos = player_chunk_pos - i as i32;
            let end_pos = player_chunk_pos + i as i32;

            for x in start_pos.x..=end_pos.x {
                for z in start_pos.z..=end_pos.z {
                    for y in start_pos.y..=end_pos.y {
                        let chunk_pos = glam::ivec3(x, y, z);
                        let mut chunk = Chunk::new(chunk_pos);
                        gen_blocks(&mut chunk, chunk_pos);
                        chunk_manager.chunk_map.insert(chunk_pos, chunk);
                    }
                }
            }

            for (chunk_pos, chunk) in chunk_manager.chunk_map.iter() {
                // Get a chunk mesh from the world to regenerate its mesh
                // If no more exist then create a new one
                let block_pos = chunk_pos.as_vec3() * CHUNK_SIZE as f32;
                if let Some((_, mut position, mut mesh, _)) = mesh_iter.next() {
                    position.0 = block_pos;
                    mesh.regenerate(&renderer.queue, chunk, &chunk_manager);
                } else {
                    let mut mesh = ChunkMesh::new(&renderer.device);
                    mesh.regenerate(&renderer.queue, chunk, &chunk_manager);
                    commands
                        .spawn()
                        .insert_bundle((WorldPosition(block_pos), mesh));
                };
            }
        }

        log::info!("Gen {} chunks.", chunk_manager.chunk_map.len());
        // Remove any remaning chunk meshes
        for (entity, _, _, _) in mesh_iter {
            commands.entity(entity).despawn();
        }
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
