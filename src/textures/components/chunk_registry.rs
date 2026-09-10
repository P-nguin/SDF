use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct ChunkRegistry {
    pub blueprints: HashMap<String, ChunkBlueprint>,
}

pub struct ChunkBlueprint {
    pub voxel_data: Vec<i8>,
    pub texture: Handle<Image>,
}