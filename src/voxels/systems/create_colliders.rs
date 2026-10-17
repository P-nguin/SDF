use bevy::prelude::*;
use avian2d::prelude::*;
use crate::voxels::components::VoxelChunkGrid;

pub fn create_voxel_colliders(
    mut commands: Commands,
    query: Query<(Entity, &VoxelChunkGrid), Changed<VoxelChunkGrid>>
) {
    for (entity, grid) in query.iter() {
        // Run greedy meshing to get optimized rectangles
        let rectangles = greedy_mesh_2d(&grid.chunk_health, grid.width, grid.height);
        
        let mut shapes = Vec::new();
        for rect in rectangles {
            // Convert to local spatial coordinates
            // This offset calculation depends on your specific visual representation.
            // If the chunk is 16x16 pixels and rendered with Sprite, 
            // the collider offset needs to match the visual offset.
            
            let center_x = rect.x as f32 + (rect.w as f32 / 2.0);
            let center_y = rect.y as f32 + (rect.h as f32 / 2.0); 
            
            // Adjust to be centered around 0,0 instead of bottom left (0,0)
            let half_width = grid.width as f32 / 2.0;
            let half_height = grid.height as f32 / 2.0;
            
            shapes.push((
                Vec2::new(center_x - half_width, -(center_y - half_height)), // Adjust based on Sprite rendering
                0.0, // rotation
                Collider::rectangle(rect.w as f32, rect.h as f32)
            ));
        }
        
        // Update the collider on the chunk entity
        if shapes.is_empty() {
            commands.entity(entity).remove::<Collider>();
        } else {
            commands.entity(entity).insert(Collider::compound(shapes));
        }
    }
}

// Simple Rect struct for our greedy mesher
struct IntRect {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

// 2D Greedy Mesher
fn greedy_mesh_2d(chunk_health: &[f32], width: usize, height: usize) -> Vec<IntRect> {
    let mut visited = vec![false; width * height];
    let mut rectangles = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            
            // If empty (health <= 0) or already visited, skip
            if chunk_health[index] <= 0.0 || visited[index] {
                continue;
            }

            // 1. Find the longest horizontal strip
            let mut w = 1;
            while x + w < width {
                let current_index = y * width + (x + w);
                if chunk_health[current_index] <= 0.0 || visited[current_index] {
                    break;
                }
                w += 1;
            }

            // 2. Find how far down this specific horizontal strip can be expanded
            let mut h = 1;
            let mut can_expand_down = true;
            
            while y + h < height && can_expand_down {
                // Check the entire strip at y + h
                for strip_x in x..(x + w) {
                    let current_index = (y + h) * width + strip_x;
                    if chunk_health[current_index] <= 0.0 || visited[current_index] {
                        can_expand_down = false;
                        break;
                    }
                }
                if can_expand_down {
                    h += 1;
                }
            }

            // 3. Mark the whole block as visited
            for dy in 0..h {
                for dx in 0..w {
                    let visited_index = (y + dy) * width + (x + dx);
                    visited[visited_index] = true;
                }
            }

            rectangles.push(IntRect { x, y, w, h });
        }
    }

    rectangles
}
