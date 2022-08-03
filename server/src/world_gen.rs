use opencuboids_common::{Chunk, CHUNK_SIZE};

pub fn gen_blocks(chunk: &mut Chunk, chunk_pos: glam::IVec3) {
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
