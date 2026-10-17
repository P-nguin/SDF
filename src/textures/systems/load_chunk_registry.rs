use bevy::prelude::*;
use bevy::asset::{LoadedFolder, LoadState};
use crate::textures::components::*;
use crate::AppState;

#[derive(Resource, Default)]
pub struct ChunkAssetsLoading(pub Handle<LoadedFolder>);

pub fn load_chunk_folder_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let folder_handle = asset_server.load_folder("textures/chunks");
    commands.insert_resource(ChunkAssetsLoading(folder_handle));
}

pub fn check_assets_ready_system(
    mut next_state: ResMut<NextState<AppState>>,
    loading: Res<ChunkAssetsLoading>,
    asset_server: Res<AssetServer>,
) {
    if asset_server.is_loaded_with_dependencies(&loading.0) {
        next_state.set(AppState::InGame);
    }
}

pub fn process_loaded_chunks_system(
    mut chunk_registry: ResMut<ChunkRegistry>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    loading: Res<ChunkAssetsLoading>,
    images: Res<Assets<Image>>,
    sprite_sheets: Res<Assets<SpriteSheetAsset>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let Some(folder) = loaded_folders.get(&loading.0) else { return; };
    
    let mut json_map = std::collections::HashMap::new();
    for handle in folder.handles.iter() {
        if let Some(path) = handle.path() {
            if path.path().extension().unwrap_or_default() == "json" {
                if let Some(stem) = path.path().file_stem().and_then(|s| s.to_str()) {
                    json_map.insert(stem.to_string(), handle.clone().typed::<SpriteSheetAsset>());
                }
            }
        }
    }

    for handle in folder.handles.iter() {
        if let Some(path) = handle.path() {
            if path.path().extension().unwrap_or_default() == "png" {
                let Some(stem) = path.path().file_stem().and_then(|s| s.to_str()) else { continue; };
                
                let image_handle = handle.clone().typed::<Image>();
                let Some(image) = images.get(&image_handle) else { continue; };

                let mut atlas_layout = None;
                let mut parts = std::collections::HashMap::new();
                let mut first_frame_rect = None;

                if let Some(json_handle) = json_map.get(stem) {
                    if let Some(sheet_asset) = sprite_sheets.get(json_handle) {
                        let sheet_data = &sheet_asset.data;
                        
                        let mut layout = TextureAtlasLayout::new_empty(UVec2::new(
                            sheet_data.meta.size.w,
                            sheet_data.meta.size.h
                        ));
                        
                        for (i, frame_data) in sheet_data.frames.iter().enumerate() {
                            if i == 0 {
                                first_frame_rect = Some((
                                    frame_data.frame.x,
                                    frame_data.frame.y,
                                    frame_data.frame.w,
                                    frame_data.frame.h,
                                ));
                            }
                            let rect = URect::new(
                                frame_data.frame.x,
                                frame_data.frame.y,
                                frame_data.frame.x + frame_data.frame.w,
                                frame_data.frame.y + frame_data.frame.h,
                            );
                            layout.add_texture(rect);
                            
                            let part_name = if let Some(ref filename) = frame_data.filename {
                                let mut name = filename.clone();
                                while name.ends_with(|c: char| c.is_digit(10) || c.is_whitespace()) {
                                    name.pop();
                                }
                                if name.is_empty() {
                                    "default".to_string()
                                } else {
                                    name
                                }
                            } else {
                                "default".to_string()
                            };
                            
                            let part = parts.entry(part_name).or_insert(SpritePart::default());
                            part.frame_indices.push(i);
                            part.frame_durations.push(frame_data.duration);
                        }
                        
                        atlas_layout = Some(texture_atlases.add(layout));
                    }
                }
                
                let (voxel_data, width, height) = generate_voxel_array_from_bevy_image(image, first_frame_rect);

                chunk_registry.blueprints.insert(
                    stem.to_string(),
                    ChunkBlueprint {
                        voxel_data,
                        width,
                        height,
                        texture: image_handle,
                        atlas_layout,
                        parts,
                    },
                );
                info!("Compiled chunk blueprint: {}", stem);
            }
        }
    }
}

fn generate_voxel_array_from_bevy_image(img: &Image, crop_rect: Option<(u32, u32, u32, u32)>) -> (Vec<i8>, usize, usize) {
    let img_width = img.texture_descriptor.size.width as usize;
    let img_height = img.texture_descriptor.size.height as usize;
    
    let (cx, cy, cw, ch) = if let Some((x, y, w, h)) = crop_rect {
        (x as usize, y as usize, w as usize, h as usize)
    } else {
        (0, 0, img_width, img_height)
    };
    
    let mut voxel_array = Vec::with_capacity(cw * ch);

    if let Some(data) = img.data.as_ref() {
        let bytes_per_pixel = data.len() / (img_width * img_height).max(1);
        
        for y in cy..(cy + ch) {
            for x in cx..(cx + cw) {
                let img_idx = y * img_width + x;
                let byte_idx = img_idx * bytes_per_pixel;
                
                if byte_idx + 3 < data.len() {
                    let alpha = data[byte_idx + 3];
                    if alpha == 0 {
                        voxel_array.push(0);
                    } else {
                        voxel_array.push(1);
                    }
                } else {
                    voxel_array.push(0);
                }
            }
        }
    } else {
        for _ in 0..(cw * ch) {
            voxel_array.push(0);
        }
    }
    
    (voxel_array, cw, ch)
}
