use serde::{
    de::{SeqAccess, Visitor},
    ser::{SerializeTuple, Serializer},
    Deserialize, Deserializer, Serialize,
};

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE.pow(3);
pub type BlockID = u8;

#[derive(Serialize, Deserialize)]
pub struct Chunk {
    #[serde(
        serialize_with = "serialize_blocks",
        deserialize_with = "deserialize_blocks"
    )]
    pub blocks: [BlockID; CHUNK_VOLUME],
    pub pos: glam::IVec3,
}

impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Chunk {{ blocks: [..], pos: {} }}", self.pos)
    }
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

    pub fn try_get_block(&self, pos: glam::UVec3) -> Option<BlockID> {
        if pos.x >= CHUNK_SIZE as u32 || pos.y >= CHUNK_SIZE as u32 || pos.z >= CHUNK_SIZE as u32 {
            None
        } else {
            Some(self.get_block(pos))
        }
    }
}

pub fn iter_3d(start: i32, end: i32) -> impl Iterator<Item = glam::IVec3> {
    (start..end).flat_map(move |x| {
        (start..end).flat_map(move |y| (start..end).map(move |z| glam::ivec3(x, y, z)))
    })
}

pub fn iter_3d_vec(start: glam::IVec3, end: glam::IVec3) -> impl Iterator<Item = glam::IVec3> {
    (start.x..end.x).flat_map(move |x| {
        (start.y..end.y).flat_map(move |y| (start.z..end.z).map(move |z| glam::ivec3(x, y, z)))
    })
}

pub fn serialize_blocks<S: Serializer>(
    blocks: &[BlockID; CHUNK_VOLUME],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut seq = serializer.serialize_tuple(blocks.len())?;
    // TODO: add run length encoding
    for block in blocks {
        seq.serialize_element(block)?
    }
    seq.end()
}

pub fn deserialize_blocks<'a, D: Deserializer<'a>>(
    deserializer: D,
) -> Result<[BlockID; CHUNK_VOLUME], D::Error> {
    struct ArrayVisitor;

    impl<'de> Visitor<'de> for ArrayVisitor {
        type Value = [BlockID; CHUNK_VOLUME];

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("array of blocks ids in a chunk")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<[BlockID; CHUNK_VOLUME], A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut blocks = [0; CHUNK_VOLUME];
            for i in 0..CHUNK_VOLUME {
                blocks[i] = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
            }
            Ok(blocks)
        }
    }

    deserializer.deserialize_tuple(CHUNK_VOLUME, ArrayVisitor)
}

fn pos_to_index(block_pos: glam::UVec3) -> usize {
    block_pos.y as usize * (CHUNK_SIZE * CHUNK_SIZE)
        + block_pos.z as usize * CHUNK_SIZE
        + block_pos.x as usize
}

pub fn in_bounds(chunk_pos: glam::IVec3, center: glam::IVec3, distance: i32) -> bool {
    let pos = chunk_pos - center;
    pos.x > -distance
        && pos.y > -distance
        && pos.z > -distance
        && pos.x < distance
        && pos.y < distance
        && pos.z < distance
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
