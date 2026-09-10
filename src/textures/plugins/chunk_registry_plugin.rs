use bevy::prelude::*;
use crate::textures::compile_chunk_folder_system;

pub struct ChunkRegistryPlugin;

impl Plugin for ChunkRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, compile_chunk_folder_system);
    }
}