use bevy::prelude::*;

#[derive(Component)]
pub struct VoxelChunk {
    pub orientation: IVec2,
}
impl VoxelChunk {
    pub fn create_voxel_chunk(chunk_type: char, chunk: Vec<i8>, orientation: IVec2, texture: Option<&Entity>) -> VoxelChunk {
        VoxelChunk { 
            orientation,
        }
    }
}
impl VoxelChunk {
    pub fn new(orientation: IVec2,) -> VoxelChunk {
        VoxelChunk {
            orientation,
        }
    }
}

#[derive(Component)]
pub struct VoxelChunkGrid {
    pub chunk_health: Vec<f32>,
    pub width: usize,
    pub height: usize,
}
impl VoxelChunkGrid {
    pub fn new(chunk_health: Vec<f32>, width: usize, height: usize) -> VoxelChunkGrid {
        VoxelChunkGrid { chunk_health, width, height }
    }
}

#[derive(Bundle)]
pub struct VoxelBundle {
    chunk: VoxelChunk,
    grid: VoxelChunkGrid,
}