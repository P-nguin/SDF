use bevy::prelude::*;
use crate::AppState;
use crate::textures::components::{SpriteSheetAsset, SpriteSheetLoader, ChunkRegistry};
use crate::textures::{load_chunk_folder_system, check_assets_ready_system, process_loaded_chunks_system, animate_sprites_system};

pub struct ChunkRegistryPlugin;

impl Plugin for ChunkRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<SpriteSheetAsset>()
            .init_asset_loader::<SpriteSheetLoader>()
            .init_resource::<ChunkRegistry>()
            .add_systems(OnEnter(AppState::Loading), load_chunk_folder_system)
            .add_systems(Update, check_assets_ready_system.run_if(in_state(AppState::Loading)))
            .add_systems(OnExit(AppState::Loading), process_loaded_chunks_system)
            .add_systems(Update, animate_sprites_system.run_if(in_state(AppState::InGame)));
    }
}