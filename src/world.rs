use std::collections::HashMap;
use std::slice;
use glam::{Vec2, Vec3};
use wgpu::{Buffer, Device};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::buffer_builder::BufferBuilder;
use crate::util::VSInput;
use crate::world;

pub const CHUNK_SIZE_X: i64 = 16;
pub const CHUNK_SIZE_Y: i64 = 16;
pub const CHUNK_SIZE_Z: i64 = 16;
pub const CHUNK_SIZE_XYZ: i64 = CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z;

pub const BLOCK_TYPE_AIR: u16 = 0;
pub const BLOCK_TYPE_STONE: u16 = 1;

#[derive(Debug, Default)]
pub struct Chunk {
    data: Vec<u16>,
    position: Position
}

impl Chunk {
    pub fn new(position: Position, filled_with: u16) -> Self {
        let mut data = (0..CHUNK_SIZE_XYZ).map(|_| filled_with).collect::<Vec<_>>();
        Self {
            position,
            data
        }
    }

    pub fn set_block(&mut self, block_type: u16, block_position: &Position) {
        self.data[(block_position.z * CHUNK_SIZE_X * CHUNK_SIZE_Y + block_position.y * CHUNK_SIZE_X + block_position.x) as usize] = block_type;
    }

    pub fn get_block(&self, block_position: &Position) -> u16 {
        self.data[(block_position.z * CHUNK_SIZE_X * CHUNK_SIZE_Y + block_position.y * CHUNK_SIZE_X + block_position.x) as usize]
    }
}

pub struct ChunkBuilder<'a> {
    chunk: &'a Chunk,
    buffer_builder: BufferBuilder
}

impl<'a> ChunkBuilder<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Self {
            chunk,
            buffer_builder: BufferBuilder::new()
        }
    }

    fn add_bottom_face(&mut self, block_position: &Position) {
        let x = block_position.x as f32;
        let y = block_position.y as f32;
        let z = block_position.z as f32;

        self.buffer_builder.add_quad(
            VSInput::new(Vec3::new(x + 1f32, y, z), Vec2::new(1.0f32, 0.0f32)),
            VSInput::new(Vec3::new(x, y, z), Vec2::new(0.0f32, 0.0f32)),
            VSInput::new(Vec3::new(x, y, z + 1f32), Vec2::new(0f32, 1f32)),
            VSInput::new(Vec3::new(x + 1f32, y, z + 1f32), Vec2::new(1.0f32, 1.0f32)),
        );
    }

    fn add_top_face(&mut self, block_position: &Position) {
        let x = block_position.x as f32;
        let y = block_position.y as f32;
        let z = block_position.z as f32;

        self.buffer_builder.add_quad(
            VSInput::new(Vec3::new(x, y + 1f32, z), Vec2::new(0.0f32, 1.0f32)),
            VSInput::new(Vec3::new(x + 1f32, y + 1f32, z), Vec2::new(1.0f32, 1.0f32)),
            VSInput::new(Vec3::new(x + 1f32, y + 1f32, z + 1f32), Vec2::new(1.0f32, 0.0f32)),
            VSInput::new(Vec3::new(x, y + 1f32, z + 1f32), Vec2::new(0f32, 0f32))
        );
    }

    fn add_west_face(&mut self, block_position: &Position) {
        let x = block_position.x as f32;
        let y = block_position.y as f32;
        let z = block_position.z as f32;

        self.buffer_builder.add_quad(
            VSInput::new(Vec3::new(x, y + 1f32, z), Vec2::new(1.0f32, 0.0f32)),
            VSInput::new(Vec3::new(x, y + 1f32, z + 1f32), Vec2::new(0.0f32, 0.0f32)),
            VSInput::new(Vec3::new(x, y, z + 1f32), Vec2::new(0.0f32, 1.0f32)),
            VSInput::new(Vec3::new(x, y, z), Vec2::new(1.0f32, 1.0f32))
        );
    }

    fn add_east_face(&mut self, block_position: &Position) {
        let x = block_position.x as f32;
        let y = block_position.y as f32;
        let z = block_position.z as f32;

        self.buffer_builder.add_quad(
            VSInput::new(Vec3::new(x + 1f32, y + 1f32, z + 1f32), Vec2::new(1.0f32, 0.0f32)),
            VSInput::new(Vec3::new(x + 1f32, y + 1f32, z), Vec2::new(0.0f32, 0.0f32)),
            VSInput::new(Vec3::new(x + 1f32, y, z), Vec2::new(0.0f32, 1.0f32)),
            VSInput::new(Vec3::new(x + 1f32, y, z + 1f32), Vec2::new(1.0f32, 1.0f32)),
        );
    }

    fn add_north_face(&mut self, block_position: &Position) {
        let x = block_position.x as f32;
        let y = block_position.y as f32;
        let z = block_position.z as f32;

        self.buffer_builder.add_quad(
            VSInput::new(Vec3::new(x, y + 1f32, z + 1f32), Vec2::new(1.0f32, 0.0f32)),
            VSInput::new(Vec3::new(x + 1f32, y + 1f32, z + 1f32), Vec2::new(0.0f32, 0.0f32)),
            VSInput::new(Vec3::new(x + 1f32, y, z + 1f32), Vec2::new(0.0f32, 1.0f32)),
            VSInput::new(Vec3::new(x, y, z + 1f32), Vec2::new(1.0f32, 1.0f32))
        );
    }

    fn add_south_face(&mut self, block_position: &Position) {
        let x = block_position.x as f32;
        let y = block_position.y as f32;
        let z = block_position.z as f32;

        self.buffer_builder.add_quad(
            VSInput::new(Vec3::new(x, y, z), Vec2::new(0.0f32, 1.0f32)),
            VSInput::new(Vec3::new(x + 1f32, y, z), Vec2::new(1.0f32, 1.0f32)),
            VSInput::new(Vec3::new(x + 1f32, y + 1f32, z), Vec2::new(1.0f32, 0.0f32)),
            VSInput::new(Vec3::new(x, y + 1f32, z), Vec2::new(0.0f32, 0.0f32)),
        );
    }

    pub fn build_mesh(&mut self, device: &Device, world: &World) -> (Buffer, Buffer, u32) {
        for x in 0..CHUNK_SIZE_X {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    let chunk_position = Position::new(x, y, z);
                    let position = Position::new(self.chunk.position.x * CHUNK_SIZE_X + x,
                                                     self.chunk.position.y * CHUNK_SIZE_Y + y,
                                                 self.chunk.position.z * CHUNK_SIZE_Z + z);
                    if self.chunk.get_block(&chunk_position) == BLOCK_TYPE_AIR {
                        continue;
                    }

                    if world.get_block(&Position::new(position.x, position.y - 1, position.z)) == BLOCK_TYPE_AIR {
                        self.add_bottom_face(&chunk_position);
                    }

                    if world.get_block(&Position::new(position.x, position.y + 1, position.z)) == BLOCK_TYPE_AIR {
                        self.add_top_face(&chunk_position);
                    }

                    if world.get_block(&Position::new(position.x - 1, position.y, position.z)) == BLOCK_TYPE_AIR {
                        self.add_west_face(&chunk_position);
                    }

                    if world.get_block(&Position::new(position.x + 1, position.y, position.z)) == BLOCK_TYPE_AIR {
                        self.add_east_face(&chunk_position);
                    }

                    if world.get_block(&Position::new(position.x, position.y, position.z + 1)) == BLOCK_TYPE_AIR {
                        self.add_north_face(&chunk_position);
                    }

                    if world.get_block(&Position::new(position.x, position.y, position.z - 1)) == BLOCK_TYPE_AIR {
                        self.add_south_face(&chunk_position);
                    }
                }
            }
        }

        self.buffer_builder.build(device)
    }
}

#[derive(Copy, Clone, Default, Debug, Hash, PartialEq, Eq)]
pub struct Position {
    pub x: i64,
    pub y: i64,
    pub z: i64
}

impl Position {
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug)]
pub struct World {
    pub chunks: HashMap<Position, Chunk>
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new()
        }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.position, chunk);
    }

    pub fn get_block(&self, position: &Position) -> u16 {
        let chunk_position = Position::new(position.x >> 4, position.y >> 4, position.z >> 4);
        if let Some(chunk) = self.chunks.get(&chunk_position) {
            let position = Position::new(position.x % CHUNK_SIZE_X, position.y % CHUNK_SIZE_Y, position.z % CHUNK_SIZE_Z);
            chunk.get_block(&position)
        } else {
            BLOCK_TYPE_AIR
        }
    }
}