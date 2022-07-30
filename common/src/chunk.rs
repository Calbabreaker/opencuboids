pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = 32 * 32 * 32;
pub type BlockID = u8;

pub struct Chunk {
    pub blocks: [BlockID; CHUNK_VOLUME],
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: [0; CHUNK_VOLUME],
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

#[macro_export]
macro_rules! loop_3d_vec {
    ($start: expr, $end: expr, $closure: expr) => {
        for x in $start.x..=$end.x {
            for y in $start.y..=$end.y {
                for z in $start.z..=$end.z {
                    let pos = glam::ivec3(x, y, z);
                    $closure(pos);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! loop_3d {
    ($range: expr, $closure: expr) => {
        for x in $range {
            for y in $range {
                for z in $range {
                    let pos = glam::ivec3(x, y, z);
                    $closure(pos);
                }
            }
        }
    };
}

fn pos_to_index(block_pos: glam::UVec3) -> usize {
    block_pos.y as usize * (CHUNK_SIZE * CHUNK_SIZE)
        + block_pos.z as usize * CHUNK_SIZE
        + block_pos.x as usize
}

pub fn in_bounds(chunk_pos: glam::IVec3, center: glam::IVec3, distance: i32) -> bool {
    let pos = chunk_pos - center;
    pos.z >= 0
        && pos.y >= 0
        && pos.z >= 0
        && pos.x <= distance
        && pos.y <= distance
        && pos.z <= distance
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
