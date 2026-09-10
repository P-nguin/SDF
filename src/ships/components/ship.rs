// In src/ships/components/ship.rs
use avian2d::prelude::*;
use bevy::prelude::*;

use crate::voxels::{VoxelChunk, VoxelChunkType};
use crate::textures::ChunkRegistry;

#[derive(Component)]
pub struct Ship;
impl Ship {
    pub fn spawn_ship(
        commands: &mut Commands,
        registry: &Res<ChunkRegistry>,
        blueprint: &[&str],
        spawn_pos: Vec2,
    ) -> Entity {
        // 1. Spawn the root entity with all baseline components (except the grid)
        let ship_entity = commands.spawn(ShipBundle {
            ship_tag: Ship,
            nav: ShipNav {
                max_speed: 200.0,
                max_accel: 50.0,
                max_turn: 3.0,
            },
            transform: Transform::from_xyz(spawn_pos.x, spawn_pos.y, 0.0),
            visibility: Visibility::Visible,
            rigid_body: RigidBody::Dynamic,
            linear_velocity: LinearVelocity::default(),
            angular_velocity: AngularVelocity::default(),
        }).id();

        // 2. Generate the grid and spawn all the child chunks
        let grid = ShipGrid::create_ship_grid(commands, registry, blueprint, ship_entity);

        // 3. Attach the populated grid memory map to the root ship
        commands.entity(ship_entity).insert(grid);

        ship_entity
    }
}

#[derive(Component)]
pub struct ShipNav {
    pub max_speed: f32,
    pub max_accel: f32,
    pub max_turn: f32,
}

#[derive(Component)]
pub struct ShipGrid {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Option<Entity>>,
}
impl ShipGrid {
    pub fn create_ship_grid(
        commands: &mut Commands,
        registry: &Res<ChunkRegistry>,
        blueprint: &[&str],
        parent_ship: Entity,
    ) -> ShipGrid {
        let height = blueprint.len();
        let width = if height > 0 { blueprint[0].len() } else { 0 };
        let mut grid_entities = vec![None; width * height];
        let chunk_size = 16.0;

        // Calculate the center offsets
        // We subtract 1.0 because a width of 5 spans indices 0 to 4 (center is 2)
        let half_width = (width as f32 - 1.0) / 2.0;
        let half_height = (height as f32 - 1.0) / 2.0;

        for (y, row) in blueprint.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                if ch == ' ' { continue; }
                let registry_key = match ch {
                    '-' => "armor",
                    _ => continue,
                };
                if let Some(chunk_blueprint) = registry.blueprints.get(registry_key) {
                    let chunk_entity = commands.spawn((
                        VoxelChunk::create_voxel_chunk(
                            VoxelChunkType::Armor,
                            chunk_blueprint.voxel_data.clone(),
                            None,
                        ),
                        Sprite {
                            image: chunk_blueprint.texture.clone(),
                            custom_size: Some(Vec2::new(chunk_size, chunk_size)),
                            ..default()
                        },
                        // Apply the offsets to center the grid on the parent entity
                        Transform::from_xyz(
                            (x as f32 - half_width) * chunk_size,
                            -(y as f32 - half_height) * chunk_size,
                            0.0
                        ),
                    )).id();
                    
                    commands.entity(parent_ship).add_child(chunk_entity);
                    
                    let index = y * width + x;
                    grid_entities[index] = Some(chunk_entity);
                } else {
                    warn!("Blueprint key '{}' not found in ChunkRegistry!", registry_key);
                }
            }
        }
        
        ShipGrid {
            width,
            height,
            grid: grid_entities,
        }
    }
}

#[derive(Bundle)]
pub struct ShipBundle {
    pub ship_tag: Ship,
    pub nav: ShipNav,
    pub transform: Transform,
    pub visibility: Visibility,

    // Physics components needed for navigation
    pub rigid_body: RigidBody,
    pub linear_velocity: LinearVelocity,
    pub angular_velocity: AngularVelocity,
}