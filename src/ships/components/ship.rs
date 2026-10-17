// In src/ships/components/ship.rs
use avian2d::prelude::*;
use bevy::prelude::*;

use crate::ships::components::*;
use crate::cells::components::*;
use crate::voxels::components::*;
use crate::textures::components::*;

#[derive(Component)]
pub struct Ship;
impl Ship {
    pub fn spawn_ship(
        commands: &mut Commands,
        chunk_registry: &Res<ChunkRegistry>,
        cell_registry: &Res<CellRegistry>,
        blueprint: &[Vec<[&str; 2]>],
        spawn_pos: Vec2,
    ) -> Entity {
        let ship_entity = commands.spawn(ShipBundle {
            ship_tag: Ship,
            nav: crate::navigation::MovementProfile {
                max_forward_thrust: 50.0,
                max_reverse_thrust: 25.0,
                max_strafe_thrust: 25.0,
                max_speed: 200.0,
                max_turn_rate: 3.0,
            },
            transform: Transform::from_xyz(spawn_pos.x, spawn_pos.y, 0.0),
            visibility: Visibility::Visible,
            rigid_body: RigidBody::Dynamic,
            linear_velocity: LinearVelocity::default(),
            angular_velocity: AngularVelocity::default(),
            targeting: crate::weapons::components::ShipTargetingComputer::default(),
            targetable: crate::controls::components::Targetable,
        }).id();

        let grid = ShipGrid::create_ship_grid(
            commands,
            chunk_registry,
            cell_registry,
            blueprint,
            ship_entity
        );
        commands.entity(ship_entity).insert(grid);
        ship_entity
    }
}

// We use MovementProfile from navigation module now

#[derive(Component)]
pub struct ShipGrid {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Option<Entity>>,
}
impl ShipGrid {
    pub fn create_ship_grid(
        commands: &mut Commands,
        chunk_registry: &Res<ChunkRegistry>,
        cell_registry: &Res<CellRegistry>,
        blueprint: &[Vec<[&str; 2]>],
        parent_ship: Entity,
    ) -> ShipGrid {
        let height = blueprint.len();
        let width = if height > 0 { blueprint[0].len() } else { 0 };
        let mut grid_entities = vec![None; width * height];
        let chunk_size = 16.0;

        let half_width = (width as f32 - 1.0) / 2.0;
        let half_height = (height as f32 - 1.0) / 2.0;

        for (y, row) in blueprint.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let cell_type = cell[0];
                let cell_dir = cell[1];

                if cell_type == " " || cell_type == "" { continue; } // Skip empty space

                let registry_key = cell_type;

                // Map the direction character to logical IVec2 and visual rotation (radians)
                let (orientation, visual_rotation) = match cell_dir {
                    "U" => (IVec2::new(0, 1), 0.0),
                    "D" => (IVec2::new(0, -1), std::f32::consts::PI),
                    "R" => (IVec2::new(1, 0), -std::f32::consts::FRAC_PI_2),
                    "L" => (IVec2::new(-1, 0), std::f32::consts::FRAC_PI_2),
                    _ => (IVec2::new(0, 1), 0.0), // Default to Up
                };

                if let Some(chunk_blueprint) = chunk_registry.blueprints.get(registry_key) {
                    let default_part = crate::textures::components::SpritePart::default();
                    let part = chunk_blueprint.parts.values().next().unwrap_or(&default_part);
                    
                    let mut atlas = None;
                    if let (Some(layout), false) = (&chunk_blueprint.atlas_layout, part.frame_indices.is_empty()) {
                        atlas = Some(TextureAtlas {
                            layout: layout.clone(),
                            index: part.frame_indices[0],
                        });
                    }

                    let chunk_entity = commands.spawn((
                        // 1. Gameplay logic component
                        Cell::from_registry(cell_type, cell_registry),
                        
                        // 2. Voxel metadata and logical state
                        CellPos { x: x as i16, y: y as i16, },
                        VoxelChunk::new(orientation),
                        {
                            let cell = Cell::from_registry(cell_type, cell_registry);
                            let chunk_health: Vec<f32> = chunk_blueprint.voxel_data.iter().map(|&v| {
                                if v == 1 { cell.health } else { 0.0 }
                            }).collect();
                            VoxelChunkGrid::new(chunk_health, chunk_blueprint.width, chunk_blueprint.height)
                        },
                        
                        CollisionLayers::new(
                            [crate::physics::GameLayer::Ship],
                            [
                                crate::physics::GameLayer::Ship,
                                crate::physics::GameLayer::Projectile,
                                crate::physics::GameLayer::Default
                            ]
                        ),

                        // 3. Visual components
                        Sprite {
                            image: chunk_blueprint.texture.clone(),
                            custom_size: Some(Vec2::new(chunk_size, chunk_size)),
                            texture_atlas: atlas,
                            ..default()
                        },
                        // Apply position offset AND visual rotation simultaneously
                        Transform::from_xyz(
                            (x as f32 - half_width) * chunk_size,
                            -(y as f32 - half_height) * chunk_size,
                            0.0
                        ).with_rotation(Quat::from_rotation_z(visual_rotation)),
                    )).id();
                    
                    if !part.frame_durations.is_empty() {
                        commands.entity(chunk_entity).insert(crate::textures::components::AnimationState {
                            timer: Timer::new(std::time::Duration::from_millis(part.frame_durations[0] as u64), TimerMode::Once),
                            current_index: 0,
                            frame_indices: part.frame_indices.clone(),
                            frame_durations: part.frame_durations.clone(),
                        });
                    }

                    commands.entity(parent_ship).add_child(chunk_entity);

                    match cell_type {
                        "+" | "*" => { commands.entity(chunk_entity).insert(PlusCell); },
                        "(" => { commands.entity(chunk_entity).insert((
                            ShieldCell, 
                            ShieldStats {
                                shield_type: "sb".to_string(),
                                shield_max_health: 100,
                                shield_health: 100,
                                shield_regen: 5,
                                shield_respawn: 5,
                                projection_distance: 5,
                                projection: None,
                                shield_thickness: 3,
                            })); },
                        "-" => { commands.entity(chunk_entity).insert(MinusCell); },
                        "T" => { commands.entity(chunk_entity).insert(crate::weapons::components::weapon::TurretBase {
                            weapon_type: "plasma".to_string(),
                            damage: 10.0,
                            max_range: 600.0,
                            firing_arc: std::f32::consts::PI * 2.0,
                            turn_speed: 5.0,
                            projectile_speed: 400.0,
                            fire_rate: 2.0,
                            barrel_entity: None,
                        }); },
                        _ => {}
                    }
                    
                    let index = y * width + x;
                    grid_entities[index] = Some(chunk_entity);

                } else {
                    warn!("Blueprint key '{}' not found in ChunkRegistry!", registry_key);
                }
            }
        }
        
        ShipGrid { width, height, grid: grid_entities }
    }
}

#[derive(Bundle)]
pub struct ShipBundle {
    pub ship_tag: Ship,
    pub nav: crate::navigation::MovementProfile,
    pub transform: Transform,
    pub visibility: Visibility,

    // Physics components needed for navigation
    pub rigid_body: RigidBody,
    pub linear_velocity: LinearVelocity,
    pub angular_velocity: AngularVelocity,
    pub targeting: crate::weapons::components::ShipTargetingComputer,
    pub targetable: crate::controls::components::Targetable,
}