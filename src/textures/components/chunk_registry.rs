use bevy::prelude::*;
use bevy::asset::{AssetLoader, LoadContext, io::Reader, AsyncReadExt};
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct SpriteSheetAsset {
    pub data: SpriteSheetData,
}

#[derive(Default, TypePath)]
pub struct SpriteSheetLoader;

impl AssetLoader for SpriteSheetLoader {
    type Asset = SpriteSheetAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let data = serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(SpriteSheetAsset { data })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

#[derive(Resource, Default)]
pub struct ChunkRegistry {
    pub blueprints: HashMap<String, ChunkBlueprint>,
}

pub struct ChunkBlueprint {
    pub voxel_data: Vec<i8>,
    pub width: usize,
    pub height: usize,
    pub texture: Handle<Image>,
    pub atlas_layout: Option<Handle<TextureAtlasLayout>>, 
    pub parts: HashMap<String, SpritePart>, 
}

#[derive(Clone, Default)]
pub struct SpritePart {
    pub frame_indices: Vec<usize>,
    pub frame_durations: Vec<u32>,
}

// for reading in the texture jsons
#[derive(Deserialize, Debug, Clone)]
pub struct SpriteSheetData {
    pub frames: Vec<FrameData>,
    pub meta: MetaData,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FrameData {
    pub filename: Option<String>,
    pub frame: RectData,
    pub duration: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RectData {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MetaData {
    pub size: SizeData,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SizeData {
    pub w: u32,
    pub h: u32,
}
// end