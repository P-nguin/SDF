use bevy::prelude::*;
use std::fs;
use std::path::Path;
use image::GenericImageView;

use crate::textures::components::*;

pub fn compile_chunk_folder_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut registry = ChunkRegistry::default();
    
    let folder_path = Path::new("assets/textures/chunks");
    compile_directory(folder_path, &asset_server, &mut texture_atlases, &mut registry);

    commands.insert_resource(registry);
}

fn compile_directory(
    dir: &Path, 
    asset_server: &Res<AssetServer>, 
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    registry: &mut ChunkRegistry) {
    let Ok(entries) = fs::read_dir(dir) else {
        warn!("Could not read directory: {:?}", dir);
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            compile_directory(&path, asset_server, texture_atlases, registry);
        } else if path.extension().and_then(|s| s.to_str()) == Some("png") {
            let voxel_data = generate_voxel_array_from_image(path.to_str().unwrap());

            let bevy_path = path.strip_prefix("assets").unwrap_or(&path);
            let texture_handle = asset_server.load(bevy_path.to_path_buf());

            let mut json_path = path.with_extension("json");
            if !json_path.exists() {
                json_path = path.with_file_name("data.json");
            }
            
            let mut atlas_layout = None;
            let mut frame_durations = None;

            if json_path.exists() {
                if let Ok(json_str) = fs::read_to_string(&json_path) {
                    if let Ok(sheet_data) = serde_json::from_str::<SpriteSheetData>(&json_str) {
                        
                        // create a new empty layout matching the image's full size
                        let mut layout = TextureAtlasLayout::new_empty(UVec2::new(
                            sheet_data.meta.size.w,
                            sheet_data.meta.size.h
                        ));
                        
                        let mut durations = Vec::new();
                        for frame_data in &sheet_data.frames {
                            let rect = URect::new(
                                frame_data.frame.x,
                                frame_data.frame.y,
                                frame_data.frame.x + frame_data.frame.w,
                                frame_data.frame.y + frame_data.frame.h,
                            );
                            layout.add_texture(rect);
                            durations.push(frame_data.duration);
                        }
                        
                        atlas_layout = Some(texture_atlases.add(layout));
                        frame_durations = Some(durations);
                    }
                }
            }

            if let Some(file_name) = path.file_stem().and_then(|n| n.to_str()) {
                registry.blueprints.insert(
                    file_name.to_string(),
                    ChunkBlueprint {
                        voxel_data,
                        texture: texture_handle,
                        atlas_layout,
                        frame_durations,
                    },
                );
                info!("Compiled chunk blueprint: {}", file_name);
            }
        }
    }
}

fn generate_voxel_array_from_image(filepath: &str) -> Vec<i8> {
    let img = image::open(filepath).expect("Failed to open image file");
    let mut voxel_array = Vec::with_capacity((img.width() * img.height()) as usize);
    
    for (_x, _y, _pixel) in img.pixels() {
        voxel_array.push(0); // always push 0 for now, even if there is visual nothing treat it as something
        // if pixel[3] == 0 {
        //     voxel_array.push(0);
        // } else {
        //     voxel_array.push(1);
        // }
    }
    
    voxel_array
}