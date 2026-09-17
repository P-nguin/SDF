// In src/ships/systems/shield_system.rs
use bevy::prelude::*;
use crate::ships::*;
use crate::cells::components::*;
use crate::textures::components::{AnimationState, ChunkRegistry};
use crate::voxels::components::VoxelGrid;

// Generates a perfectly symmetrical U-shaped shield blueprint.
fn generate_shield_blueprint(distance: i32, thickness: i32) -> Vec<Vec<&'static str>> {
    let actual_thickness = thickness.max(1);

    if distance <= 0 {
        return vec![vec!["sb"; actual_thickness as usize]];
    }

    let half_width = distance; 
    let grid_width = (half_width * 2 + 1) as usize;
    let max_pullback = distance;
    let grid_height = (max_pullback + actual_thickness) as usize;

    let mut blueprint = vec![vec![" "; grid_width]; grid_height];

    let mut prev_y: Option<i32> = None;

    // ONLY loop from the center (0) to the right edge (half_width)
    for x in 0..=half_width {
        // Calculate math for the right side
        let y = ((x as f32).powi(2) * (max_pullback as f32) / (half_width as f32).powi(2)).round() as i32;

        let mut draw_y_start = y;
        let draw_y_end = y;

        // Fill steep vertical gaps
        if let Some(py) = prev_y {
            if y > py + 1 {
                draw_y_start = py + 1; 
            }
        }

        // Draw the blocks and apply thickness
        for fill_y in draw_y_start..=draw_y_end {
            for dy in 0..actual_thickness {
                let final_y = fill_y + dy;
                if final_y >= 0 && (final_y as usize) < grid_height {
                    
                    // MIRROR MAGIC: Draw the right side
                    let right_x = (half_width + x) as usize;
                    blueprint[final_y as usize][right_x] = "sb";
                    
                    // MIRROR MAGIC: Draw the exact same thing on the left side
                    let left_x = (half_width - x) as usize;
                    blueprint[final_y as usize][left_x] = "sb";
                }
            }
        }
        prev_y = Some(y);
    }

    blueprint
}

pub fn shield_builder_system(
    mut commands: Commands,
    chunk_registry: Res<ChunkRegistry>,
    mut projectors: Query<(Entity, &mut ShieldStats), Changed<ShieldStats>>, 
) {
    let chunk_size = 16.0;

    for (generator_entity, mut stats) in projectors.iter_mut() {
        if let Some(old_entity) = stats.projection {
            if let Ok(mut entity_commands) = commands.get_entity(old_entity) {
                entity_commands.despawn();
            }
        }

        let blueprint = generate_shield_blueprint(stats.projection_distance, stats.shield_thickness);

        let grid = VoxelGrid::create_voxel_grid(
            &blueprint,
            |x, y, cell_type, local_x, local_y| {
                if *cell_type == " " || *cell_type == "" { return None; }
                
                if let Some(chunk_blueprint) = chunk_registry.blueprints.get(*cell_type) {
                    let chunk_entity = commands.spawn((
                        Sprite {
                            image: chunk_blueprint.texture.clone(),
                            custom_size: Some(Vec2::new(chunk_size, chunk_size)),
                            texture_atlas: chunk_blueprint.atlas_layout.as_ref().map(|layout| {
                                TextureAtlas { layout: layout.clone(), index: 0 }
                            }),
                            ..default()
                        },
                        Transform::from_xyz(local_x, local_y, 0.0),
                    )).id();
                    
                    if let Some(durations) = &chunk_blueprint.frame_durations {
                        if !durations.is_empty() {
                            commands.entity(chunk_entity).insert(AnimationState {
                                timer: Timer::new(std::time::Duration::from_millis(durations[0] as u64), TimerMode::Once),
                                current_frame: 0,
                                frame_durations: durations.clone(),
                            });
                        }
                    }
                    Some(chunk_entity)
                } else {
                    None
                }
            }
        );

        let bubble_entities: Vec<Entity> = grid.grid.iter().filter_map(|&e| e).collect();
        let local_y_offset = stats.projection_distance as f32 * chunk_size;

        let projection_entity = commands.spawn(ShieldProjectionBundle {
            marker: ShieldProjection,
            grid, 
            transform: Transform::from_xyz(0.0, local_y_offset, 1.0),
            visibility: Visibility::Visible,
        }).id();

        commands.entity(projection_entity).add_children(&bubble_entities);
        commands.entity(generator_entity).add_child(projection_entity);
        stats.projection = Some(projection_entity);
    }
}