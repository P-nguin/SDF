use bevy::prelude::*;

pub const SDF_GRID_SIZE: i32 = 64;
pub const SDF_WORLD_SIZE: f32 = 16.0;

#[derive(Component)]
pub struct SDFChunk {
    pub distances: Vec<f32>,
    pub material: Handle<StandardMaterial>,
    pub data_material: Handle<StandardMaterial>,
    
    // pub chunk_index: IVec3,
}

#[derive(Component)]
pub struct GeneratedShipMesh {
    pub ship: Entity,
}

#[derive(Component)]
pub struct GenerateMesh;
