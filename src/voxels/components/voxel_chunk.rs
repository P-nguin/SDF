use bevy::prelude::*;

#[derive(Component)]
pub struct VoxelChunk {
    chunk_type: VoxelChunkType,
    chunk: Vec<i8>,
    texture: Option<Entity>,
}
impl VoxelChunk {
    pub fn create_voxel_chunk(chunk_type: VoxelChunkType, chunk: Vec<i8>, texture: Option<&Entity>) -> VoxelChunk {
        VoxelChunk { 
            chunk_type, 
            chunk, 
            texture: texture.copied()
        }
    }
}


#[derive(Component)]
pub struct VoxelChunkTexture {
    texture: Handle<Image>
}
impl VoxelChunkTexture {
    pub fn create_voxel_chunk_texture(texture: Handle<Image>) -> VoxelChunkTexture {
        VoxelChunkTexture {
            texture
        }
    }
}

pub enum VoxelChunkType {
    Armor,
}