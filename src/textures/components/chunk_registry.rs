use bevy::prelude::*;
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Resource, Default)]
pub struct ChunkRegistry {
    pub blueprints: HashMap<String, ChunkBlueprint>,
}

pub struct ChunkBlueprint {
    pub voxel_data: Vec<i8>,
    pub texture: Handle<Image>,
    // Optional atlas layout for sprite sheets
    pub atlas_layout: Option<Handle<TextureAtlasLayout>>, 
    // Optional frame durations to drive your animation system later
    pub frame_durations: Option<Vec<u32>>, 
}

// for reading in the texture jsons
#[derive(Deserialize)]
pub struct SpriteSheetData {
    pub frames: Vec<FrameData>,
    pub meta: MetaData,
}

#[derive(Deserialize)]
pub struct FrameData {
    pub frame: RectData,
    pub duration: u32,
}

#[derive(Deserialize)]
pub struct RectData {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Deserialize)]
pub struct MetaData {
    pub size: SizeData,
}

#[derive(Deserialize)]
pub struct SizeData {
    pub w: u32,
    pub h: u32,
}
// end