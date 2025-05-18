use crate::{MeshAsset, Vertex};

pub type Block = u8;
#[derive(Debug)]
pub struct Chunk {
    pub blocks: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    pub mesh: Option<MeshAsset>,
    pub pos: (i32, i32, i32),
    pub dirty: bool,
}
pub const CHUNK_SIZE: usize = 16;

impl Chunk {
    pub fn new(pos: (i32, i32, i32)) -> Self {
        Self {
            blocks: [[[1; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            pos,
            mesh: None,
            dirty: true,
        }
    }
    pub fn flat(pos: (i32, i32, i32)) -> Chunk {
        let mut chunk = Chunk::new(pos);
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                chunk.set_block(x, 0, z, 1);
                for y in 1..CHUNK_SIZE {
                    chunk.set_block(x, y, z, 0);
                }
            }
        }
        chunk
    }
    pub fn cube(pos: (i32, i32, i32)) -> Chunk {
        let mut chunk = Chunk::new(pos);
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    chunk.set_block(x, y, z, 0);
                }
            }
        }
        for x in 6..10 {
            for y in 6..10 {
                for z in 6..10 {
                    chunk.set_block(x, y, z, 1);
                }
            }
        }
        chunk
    }
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            self.blocks[x][y][z] = block;
            self.dirty = true;
        }
    }

    pub fn get_block(&self, x: isize, y: isize, z: isize) -> Block {
        if x >= 0
            && x < CHUNK_SIZE as isize
            && y >= 0
            && y < CHUNK_SIZE as isize
            && z >= 0
            && z < CHUNK_SIZE as isize
        {
            self.blocks[x as usize][y as usize][z as usize]
        } else {
            0
        }
    }

    pub fn build_chunk_mesh(&self) -> MeshAsset {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let _index_offset = 0u32;

        // (normal, tangent, [4 vertex positions])
        const FACES: [([f32; 3], [f32; 3], [[f32; 3]; 4], [[f32; 2]; 4]); 6] = [
            // +X
            (
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                [
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [1.0, 1.0, 1.0],
                    [1.0, 0.0, 1.0],
                ],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
            // -X
            (
                [-1.0, 0.0, 0.0],
                [0.0, 0.0, -1.0],
                [
                    [0.0, 0.0, 1.0],
                    [0.0, 1.0, 1.0],
                    [0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0],
                ],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
            // +Y
            (
                [0.0, 1.0, 0.0],
                [1.0, 0.0, 0.0],
                [
                    [0.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0],
                    [1.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                ],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
            // -Y
            (
                [0.0, -1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 0.0, 1.0],
                    [0.0, 0.0, 1.0],
                ],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
            // +Z
            (
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
                [
                    [0.0, 0.0, 1.0],
                    [1.0, 0.0, 1.0],
                    [1.0, 1.0, 1.0],
                    [0.0, 1.0, 1.0],
                ],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
            // -Z
            (
                [0.0, 0.0, -1.0],
                [-1.0, 0.0, 0.0],
                [
                    [0.0, 1.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                ],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
        ];

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let block = self.blocks[x][y][z];
                    if block == 0 {
                        continue;
                    } // "air"

                    let world_pos = [
                        x as f32 + self.pos.0 as f32 * CHUNK_SIZE as f32,
                        y as f32 + self.pos.1 as f32 * CHUNK_SIZE as f32,
                        z as f32 + self.pos.2 as f32 * CHUNK_SIZE as f32,
                    ];

                    for (face_idx, (normal, tangent, corners, uvs)) in FACES.iter().enumerate() {
                        let (nx, ny, nz) = match face_idx {
                            0 => (1, 0, 0),
                            1 => (-1, 0, 0),
                            2 => (0, 1, 0),
                            3 => (0, -1, 0),
                            4 => (0, 0, 1),
                            5 => (0, 0, -1),
                            _ => (0, 0, 0),
                        };

                        let neighbor = {
                            let nx = x.wrapping_add_signed(nx);
                            let ny = y.wrapping_add_signed(ny);
                            let nz = z.wrapping_add_signed(nz);
                            if nx < CHUNK_SIZE && ny < CHUNK_SIZE && nz < CHUNK_SIZE {
                                self.blocks[nx][ny][nz]
                            } else {
                                0 // "air"
                            }
                        };

                        if neighbor != 0 {
                            continue;
                        }

                        let color = match block {
                            1 => [0.5, 0.5, 0.5], // stone
                            _ => [1.0, 1.0, 1.0], // default
                        };

                        let base = vertices.len() as u32;
                        for i in 0..4 {
                            vertices.push(Vertex {
                                position: [
                                    world_pos[0] + corners[i][0],
                                    world_pos[1] + corners[i][1],
                                    world_pos[2] + corners[i][2],
                                ],
                                color,
                                tex_coords: uvs[i],
                                normal: *normal,
                                tangent: *tangent,
                            });
                        }
                        indices.extend_from_slice(&[
                            base,
                            base + 1,
                            base + 2,
                            base,
                            base + 2,
                            base + 3,
                        ]);
                    }
                }
            }
        }
        MeshAsset { vertices, indices }
    }
}
