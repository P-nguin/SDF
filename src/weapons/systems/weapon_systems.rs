use bevy::prelude::*;
use avian2d::prelude::*;
use crate::weapons::components::*;
use crate::projectiles::*;
use crate::voxels::components::VoxelGrid;
use crate::textures::components::ChunkRegistry;

pub fn turret_builder_system(
    mut commands: Commands,
    chunk_registry: Res<ChunkRegistry>,
    mut bases: Query<(Entity, &mut TurretBase), Changed<TurretBase>>,
) {
    let chunk_size = 16.0;

    for (base_entity, mut base) in bases.iter_mut() {
        if let Some(old_barrel) = base.barrel_entity {
            if let Ok(mut entity_commands) = commands.get_entity(old_barrel) {
                entity_commands.despawn();
            }
        }

        // A simple 1x2 column pointing UP as the barrel structure
        let blueprint = vec![
            vec!["-"], 
            vec!["-"]
        ];

        let grid = VoxelGrid::create_voxel_grid(
            &blueprint,
            |_x, _y, cell_type, local_x, local_y| {
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
                        Transform::from_xyz(local_x, local_y, 0.1), // Slightly above base to be visible
                    )).id();
                    
                    Some(chunk_entity)
                } else {
                    None
                }
            }
        );

        let barrel_entities: Vec<Entity> = grid.grid.iter().filter_map(|&e| e).collect();

        // Spawn a VoxelGrid wrapper holding the WeaponBundle
        let barrel_wrapper = commands.spawn((
            grid,
            Transform::from_xyz(0.0, 0.0, 1.0), // Base pivot stays localized effectively
            Visibility::Inherited,
            WeaponBundle {
                turret: Turret {
                    weapon_type: base.weapon_type.clone(),
                    damage: base.damage,
                    max_range: base.max_range,
                    firing_arc: base.firing_arc,
                    turn_speed: base.turn_speed,
                    projectile_speed: base.projectile_speed,
                },
                target_control: TargetControl::new(),
                fire_control: FireControl::new(base.fire_rate),
            },
        )).id();
        
        commands.entity(barrel_wrapper).add_children(&barrel_entities);
        commands.entity(base_entity).add_child(barrel_wrapper);
        base.barrel_entity = Some(barrel_wrapper);
    }
}

pub fn target_control_system(
    time: Res<Time>,
    // The target control needs its own Transform to rotate
    mut turrets: Query<(&mut Transform, &Turret, &mut TargetControl)>,
    targets: Query<&Transform, (Without<TargetControl>, Without<Turret>)>, // Quick way to get targets without hitting itself
) {
    let dt = time.delta_secs();

    for (mut transform, turret, mut target_control) in turrets.iter_mut() {
        if let Some(target_entity) = target_control.current_target {
            if let Ok(target_transform) = targets.get(target_entity) {
                // Calculate the direction to the target
                let direction = target_transform.translation.truncate() - transform.translation.truncate();
                // The angle we WANT to be at
                // Note: subtract PI/2 because Bevy 2D sprites usually assume UP (Vec3::Y) is 0 rotation.
                // atan2(1, 0) is PI/2 (UP). So PI/2 - PI/2 = 0 radians.
                let desired_angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;

                // For simplicity, we just use the current translation to get the angle in 2d.
                // In Bevy 2D, rotations are around the Z axis.
                let (axis, current_angle) = transform.rotation.to_axis_angle();
                let mut current_z_angle = if axis.z < 0.0 { -current_angle } else { current_angle };

                // Get shortest angular distance
                let mut angle_diff = desired_angle - current_z_angle;
                while angle_diff > std::f32::consts::PI { angle_diff -= std::f32::consts::PI * 2.0; }
                while angle_diff < -std::f32::consts::PI { angle_diff += std::f32::consts::PI * 2.0; }

                // Step towards the desired angle
                let max_turn = turret.turn_speed * dt;
                
                if angle_diff.abs() <= max_turn {
                    current_z_angle = desired_angle;
                    target_control.is_locked_on = true;
                } else {
                    current_z_angle += max_turn * angle_diff.signum();
                    target_control.is_locked_on = false; // Still turning
                }

                // Apply rotation
                transform.rotation = Quat::from_rotation_z(current_z_angle);
                target_control.current_aim_angle = current_z_angle;
            } else {
                // Target no longer exists or lacks transform
                target_control.current_target = None;
                target_control.is_locked_on = false;
            }
        } else {
            target_control.is_locked_on = false;
        }
    }
}

pub fn fire_control_system(
    mut commands: Commands,
    time: Res<Time>,
    // Fetch the GlobalTransform to accurately spawn the projectile in world coordinates
    mut weapons: Query<(&GlobalTransform, &Turret, &TargetControl, &mut FireControl)>,
) {
    let dt = time.delta();
    for (global_transform, turret, target_control, mut fire_control) in weapons.iter_mut() {
        fire_control.reload_timer.tick(dt);

        if target_control.is_locked_on && fire_control.reload_timer.just_finished() {
            let transform = global_transform.compute_transform();
            
            // Assume 0 rotation natively points UP (Vec3::Y), so we compute local Up mapped to world:
            let direction = transform.rotation * Vec3::Y;
            let velocity = direction.truncate() * turret.projectile_speed;

            if turret.weapon_type == "plasma" {
                commands.spawn(PlasmaProjectileBundle {
                    plasma_projectile_tag: PlasmaProjectile,
                    plasma_projectile_stats: PlasmaProjectileStats {
                        damage: turret.damage as i32,
                        energy: 100,
                        energy_loss_rate: 10,
                    },
                    sprite: Sprite {
                        color: Color::srgb(0.0, 0.0, 1.0),
                        custom_size: Some(Vec2::new(4.0, 8.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(transform.translation)
                                .with_rotation(transform.rotation),
                    visibility: Visibility::Visible,
                    rigid_body: RigidBody::Dynamic,
                    collider: Collider::circle(2.0),
                    sensor: Sensor,
                    velocity: LinearVelocity(velocity),
                });
            } else {
                info!("Unknown weapon_type: {}", turret.weapon_type);
            }
        }
    }
}
