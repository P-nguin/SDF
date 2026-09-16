use bevy::prelude::*;
use crate::textures::{compile_chunk_folder_system, animate_sprites_system};

pub struct ChunkRegistryPlugin;

impl Plugin for ChunkRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, compile_chunk_folder_system);
        app.add_systems(Update, animate_sprites_system);
    }
}