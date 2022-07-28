pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = 32 * 32 * 32;
pub type BlockID = u8;

pub struct Chunk {
    pub blocks: [BlockID; CHUNK_VOLUME],
    pub pos: glam::IVec3,
}

impl Chunk {
    pub fn new(pos: glam::IVec3) -> Self {
        Self {
            blocks: [0; CHUNK_VOLUME],
            pos,
        }
    }
}

impl Chunk {
    pub fn set_block(&mut self, pos: glam::UVec3, id: BlockID) {
        self.blocks[pos_to_index(pos)] = id;
    }

    pub fn get_block(&self, pos: glam::UVec3) -> BlockID {
        self.blocks[pos_to_index(pos)]
    }
}

fn pos_to_index(block_pos: glam::UVec3) -> usize {
    block_pos.y as usize * (CHUNK_SIZE * CHUNK_SIZE)
        + block_pos.z as usize * CHUNK_SIZE
        + block_pos.x as usize
}

/// Uses a direction index
pub const DIRECTION_TO_VECTOR: &[glam::IVec3] = &[
    glam::ivec3(0, 0, 1),  // North
    glam::ivec3(0, 0, -1), // South
    glam::ivec3(1, 0, 0),  // East
    glam::ivec3(-1, 0, 0), // West
    glam::ivec3(0, 1, 0),  // Top
    glam::ivec3(0, -1, 0), // Bottom
];
