use bevy::prelude::*;
use crate::voxels::components::VoxelChunkGrid;
use crate::cells::components::Cell;

#[derive(Component)]
pub struct DamageTextures {
    pub original: Handle<Image>,
}

pub fn update_visual_damage_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut query: Query<(Entity, &VoxelChunkGrid, &Cell, &mut Sprite, Option<&DamageTextures>), Changed<VoxelChunkGrid>>
) {
    for (entity, grid, cell, mut sprite, damage_textures_opt) in query.iter_mut() {
        let max_health = cell.max_health;
        if max_health <= 0.0 { continue; }
        
        info!("Updating visual damage for chunk {:?}", entity);

        let (original_handle, unique_handle) = if let Some(dt) = damage_textures_opt {
            (dt.original.clone(), sprite.image.clone())
        } else {
            let original_handle = sprite.image.clone();
            // Create a clone
            if let Some(original_image) = images.get(&original_handle) {
                let unique_image = original_image.clone();
                let unique_handle = images.add(unique_image);
                sprite.image = unique_handle.clone();
                commands.entity(entity).insert(DamageTextures {
                    original: original_handle.clone(),
                });
                (original_handle, unique_handle)
            } else {
                continue;
            }
        };

        let mut new_data = if let Some(orig) = images.get(&original_handle) {
            if let Some(data) = orig.data.as_ref() {
                data.clone()
            } else {
                info!("Original image has no data!");
                continue;
            }
        } else {
            info!("Original handle not found in images!");
            continue;
        };

        if let Some(mut unique_img) = images.get_mut(&unique_handle) {
            let img_width = unique_img.texture_descriptor.size.width as usize;
            let img_height = unique_img.texture_descriptor.size.height as usize;
            let bytes_per_pixel = 4;
            
            if new_data.len() != img_width * img_height * bytes_per_pixel {
                info!("Size mismatch! Data len: {}, expected: {}", new_data.len(), img_width * img_height * bytes_per_pixel);
                continue;
            }
            
            let mut darkened_pixels = 0;

            for y in 0..grid.height {
                for x in 0..grid.width {
                    let grid_idx = y * grid.width + x;
                    let health = grid.chunk_health[grid_idx];
                    let health_percent = (health / max_health).clamp(0.0, 1.0);

                    let mut frame_y = y;
                    while frame_y < img_height {
                        let mut frame_x = x;
                        while frame_x < img_width {
                            let img_idx = frame_y * img_width + frame_x;
                            let byte_idx = img_idx * bytes_per_pixel;
                            
                            if health <= 0.0 {
                                new_data[byte_idx + 3] = 0; // Alpha
                            } else {
                                // Darken RGB aggressively for visibility
                                // square the percent to make it darker faster
                                let aggressive_percent = health_percent * health_percent;
                                new_data[byte_idx] = (new_data[byte_idx] as f32 * aggressive_percent) as u8;
                                new_data[byte_idx + 1] = (new_data[byte_idx + 1] as f32 * aggressive_percent) as u8;
                                new_data[byte_idx + 2] = (new_data[byte_idx + 2] as f32 * aggressive_percent) as u8;
                                if health_percent < 1.0 { darkened_pixels += 1; }
                            }
                            
                            frame_x += grid.width;
                        }
                        frame_y += grid.height;
                    }
                }
            }
            info!("Updated texture data! Darkened {} pixels", darkened_pixels);
            unique_img.data = Some(new_data);
        } else {
            info!("Unique handle not found in images!");
        }
    }
}
