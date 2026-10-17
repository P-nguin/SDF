use bevy::prelude::*;
use bevy::ecs::relationship::Relationship;
use avian2d::prelude::*;
use crate::projectiles::components::PlasmaProjectileStats;
use crate::voxels::components::VoxelChunkGrid;

pub fn hit_detection_system(
    mut commands: Commands,
    projectiles: Query<(Entity, &PlasmaProjectileStats, Option<&CollidingEntities>, &GlobalTransform, &LinearVelocity)>,
    mut chunks: Query<(&mut VoxelChunkGrid, &GlobalTransform)>,
    parents: Query<&ChildOf>,
) {
    for (proj_entity, proj_stats, colliding_entities_opt, proj_transform, velocity) in projectiles.iter() {
        if colliding_entities_opt.is_none() {
            println!("Projectile {:?} HAS NO CollidingEntities!", proj_entity);
            continue;
        }
        let colliding_entities = colliding_entities_opt.unwrap();
        let mut despawn = false;
        
        if !colliding_entities.is_empty() {
            println!("Projectile {:?} colliding with {} entities", proj_entity, colliding_entities.len());
        }
        for &hit_entity in colliding_entities.iter() {
            println!("Projectile {:?} hit entity {:?}", proj_entity, hit_entity);
            // Check if the hit chunk belongs to the projectile's source ship
            if let Some(source_ship) = proj_stats.source_ship {
                let mut current_ancestor = hit_entity;
                let mut is_own_ship = false;
                
                // Climb up the ancestor tree using ChildOf
                while let Ok(child_of) = parents.get(current_ancestor) {
                    if child_of.get() == source_ship {
                        is_own_ship = true;
                        break;
                    }
                    current_ancestor = child_of.get();
                }

                if is_own_ship {
                    println!("Projectile {:?} hit own ship {:?} (chunk {:?}), ignoring", proj_entity, source_ship, hit_entity);
                    continue;
                } else {
                    println!("Projectile {:?} hit different ship.", proj_entity);
                }
            } else {
                println!("Projectile {:?} has no source ship set", proj_entity);
            }

            if let Ok((mut grid, chunk_transform)) = chunks.get_mut(hit_entity) {
                println!("Projectile {:?} hit grid {:?}", proj_entity, hit_entity);
                despawn = true;

                // We trace a line from the projectile's position last frame (approx) to its current position
                let world_pt = proj_transform.translation().truncate();
                let prev_world_pt = world_pt - velocity.0 * (1.0 / 60.0); // Assuming ~60hz physics
                
                println!("world_pt: {:?}, prev_world_pt: {:?}, chunk_world: {:?}", world_pt, prev_world_pt, chunk_transform.translation());
                
                // Convert to chunk local space
                let local_pt = chunk_transform.affine().inverse().transform_point3(world_pt.extend(0.0)).truncate();
                let prev_local_pt = chunk_transform.affine().inverse().transform_point3(prev_world_pt.extend(0.0)).truncate();

                let half_width = grid.width as f32 / 2.0;
                let half_height = grid.height as f32 / 2.0;

                // Basic clamping for fallback
                let clamped_x = local_pt.x.clamp(-half_width, half_width - 0.001);
                let clamped_y = local_pt.y.clamp(-half_height, half_height - 0.001);
                let mut hit_voxel_x = (clamped_x + half_width).floor() as isize;
                let mut hit_voxel_y = (half_height - clamped_y).floor() as isize;

                // Raycast forward from prev_local_pt to find the FIRST solid voxel hit
                let mut ray_dir = local_pt - prev_local_pt;
                if ray_dir.length_squared() < 0.0001 {
                    // Fallback direction based on velocity
                    let local_vel = chunk_transform.affine().inverse().transform_vector3(velocity.0.extend(0.0)).truncate();
                    ray_dir = if local_vel.length_squared() > 0.0001 { local_vel.normalize() } else { Vec2::X };
                } else {
                    ray_dir = ray_dir.normalize();
                }
                
                let mut found = false;
                let max_dist: f32 = 32.0; // Trace far enough to cross the whole chunk
                let steps = (max_dist * 2.0).ceil() as usize; // Step every 0.5 units
                
                for i in 0..=steps {
                    let p = prev_local_pt + ray_dir * (i as f32 * 0.5);
                    
                    if p.x >= -half_width && p.x < half_width && p.y >= -half_height && p.y < half_height {
                        let vx = (p.x + half_width).floor() as isize;
                        let vy = (half_height - p.y).floor() as isize;
                        
                        if vx >= 0 && vx < grid.width as isize && vy >= 0 && vy < grid.height as isize {
                            let idx = (vy as usize) * grid.width + (vx as usize);
                            if grid.chunk_health[idx] > 0.0 {
                                // Found the true entry point
                                hit_voxel_x = vx;
                                hit_voxel_y = vy;
                                found = true;
                                break;
                            }
                        }
                    }
                }

                if !found {
                    // It passed through empty space (a hole) or hasn't reached the solid part yet.
                    // Do not apply damage and do NOT despawn!
                    despawn = false;
                    continue;
                }

                println!("Hit local_pt: {:?}, voxel coord: ({}, {})", local_pt, hit_voxel_x, hit_voxel_y);

                // Apply damage with a small radius
                let damage = proj_stats.damage as f32;
                let radius = 2; // small blast radius

                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let vx = hit_voxel_x + dx;
                        let vy = hit_voxel_y + dy;
                        
                        if vx >= 0 && vx < grid.width as isize && vy >= 0 && vy < grid.height as isize {
                            // Distance check for circle
                            if (dx*dx + dy*dy) as f32 <= (radius*radius) as f32 {
                                let idx = (vy as usize) * grid.width + (vx as usize);
                                if grid.chunk_health[idx] > 0.0 {
                                    grid.chunk_health[idx] -= damage;
                                    println!("Damaged voxel at ({}, {}), new health: {}", vx, vy, grid.chunk_health[idx]);
                                    if grid.chunk_health[idx] < 0.0 {
                                        grid.chunk_health[idx] = 0.0;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if despawn {
            if let Ok(mut entity_commands) = commands.get_entity(proj_entity) {
                entity_commands.despawn();
            }
        }
    }
}
