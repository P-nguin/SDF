use bevy::prelude::*;
use avian3d::prelude::*;

use crate::voxel::components::*;

#[derive(Resource, Default)]
pub struct DestructionMode(pub bool);

pub fn toggle_destruction_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut destruction_mode: ResMut<DestructionMode>,
) {
    if keyboard.just_pressed(KeyCode::KeyB) {
        destruction_mode.0 = !destruction_mode.0;
        info!("Destruction Mode: {}", if destruction_mode.0 { "ON" } else { "OFF" });
    }
}

pub fn handle_voxel_destruction(
    mut commands: Commands,
    destruction_mode: Res<DestructionMode>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    spatial_query: SpatialQuery,
    mut chunk_query: Query<(&GlobalTransform, &mut SDFChunk), Without<Camera3d>>,
    _entity_query: Query<Entity>,
) {
    if !destruction_mode.0 || !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok((camera, camera_transform)) = cameras.single() else { return; };

    // 1. Generate a ray from the camera through the cursor
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return; };

    // 2. Perform Avian3D spatial raycast against the world
    let max_distance = 1000.0;
    if let Some(hit) = spatial_query.cast_ray(
        ray.origin,
        ray.direction,
        max_distance,
        true, // solid objects
        &SpatialQueryFilter::default(),
    ) {
        let hit_entity = hit.entity;
        let hit_point = ray.origin + ray.direction * hit.distance;

        // 3. Check if the hit entity has our SdfChunk data
        if let Ok((global_transform, mut chunk)) = chunk_query.get_mut(hit_entity) {
            // Transform world hit point into the ship's local coordinate space
            let transform = global_transform.to_matrix();
            let inverse_transform = transform.inverse();
            let local_hit_point = inverse_transform.transform_point3(hit_point);

            // Map local point into voxel grid indices (0 to SDF_GRID_SIZE)
            let voxel_size = SDF_WORLD_SIZE / (SDF_GRID_SIZE - 1) as f32;
            let grid_center = SDF_WORLD_SIZE * 0.5;
            
            // Shift from center-origin coords to positive grid index coords
            let voxel_space_pos = (local_hit_point + Vec3::splat(grid_center)) / voxel_size;
            let hit_voxel = IVec3::new(
                voxel_space_pos.x.round() as i32,
                voxel_space_pos.y.round() as i32,
                voxel_space_pos.z.round() as i32,
            );

            // 4. Carve out a sphere in the local distances grid
            let radius_voxels = 5; // Size of the explosion
            carve_sphere_at_hit(&mut chunk.distances, SDF_GRID_SIZE as usize, hit_voxel, radius_voxels);

            // 5. Trigger the re-bake component so your compute pipeline picks it up
            commands.entity(hit_entity).insert(GenerateMesh);
            
            info!("Blasted voxel chunk at local grid coords: {:?}", hit_voxel);
        }
    }
}

fn carve_sphere_at_hit(
    distances: &mut Vec<f32>,
    grid_size: usize,
    hit_voxel: IVec3,
    radius_voxels: i32,
) {
    let r_squared = (radius_voxels as f32).powi(2);

    let min_x = (hit_voxel.x - radius_voxels).max(0);
    let max_x = (hit_voxel.x + radius_voxels).min((grid_size - 1) as i32);
    let min_y = (hit_voxel.y - radius_voxels).max(0);
    let max_y = (hit_voxel.y + radius_voxels).min((grid_size - 1) as i32);
    let min_z = (hit_voxel.z - radius_voxels).max(0);
    let max_z = (hit_voxel.z + radius_voxels).min((grid_size - 1) as i32);

    for z in min_z..=max_z {
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let current_pos = IVec3::new(x, y, z);
                let dist_sq = (current_pos - hit_voxel).as_vec3().length_squared();

                if dist_sq <= r_squared {
                    // Cast x, y, z to usize so Rust accepts them as indices for the vector
                    let index = z as usize * grid_size * grid_size + y as usize * grid_size + x as usize;
                    let dist_to_surface = dist_sq.sqrt() - radius_voxels as f32;
                    
                    distances[index] = distances[index].max(-dist_to_surface);
                }
            }
        }
    }
}
