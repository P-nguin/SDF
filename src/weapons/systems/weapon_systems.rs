use bevy::prelude::*;
use bevy::ecs::relationship::Relationship;
use avian2d::prelude::*;
use crate::weapons::components::*;
use crate::weapons::targeting::*;
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

        // Use "T" to refer to the turret chunk blueprint
        let blueprint = vec![vec!["T"]];
        let mut turret_entity_opt: Option<Entity> = None;

        let grid = VoxelGrid::create_voxel_grid(
            &blueprint,
            |_x, _y, cell_type, local_x, local_y| {
                if *cell_type == " " || *cell_type == "" { return None; }
                
                if let Some(chunk_blueprint) = chunk_registry.blueprints.get(*cell_type) {
                    let default_part = crate::textures::components::SpritePart::default();
                    let base_part = chunk_blueprint.parts.get("Base").unwrap_or_else(|| chunk_blueprint.parts.values().next().unwrap_or(&default_part));
                    
                    let mut base_atlas = None;
                    if let (Some(layout), false) = (&chunk_blueprint.atlas_layout, base_part.frame_indices.is_empty()) {
                        base_atlas = Some(TextureAtlas { layout: layout.clone(), index: base_part.frame_indices[0] });
                    }

                    let base_sprite = commands.spawn((
                        Sprite {
                            image: chunk_blueprint.texture.clone(),
                            custom_size: Some(Vec2::new(chunk_size, chunk_size)),
                            texture_atlas: base_atlas,
                            ..default()
                        },
                        Transform::from_xyz(local_x, local_y, 0.1),
                    )).id();
                    
                    if !base_part.frame_durations.is_empty() {
                       commands.entity(base_sprite).insert(crate::textures::components::AnimationState { 
                           timer: Timer::new(std::time::Duration::from_millis(base_part.frame_durations[0] as u64), TimerMode::Once),
                           current_index: 0,
                           frame_indices: base_part.frame_indices.clone(),
                           frame_durations: base_part.frame_durations.clone(),
                       });
                    }

                    let turret_part = chunk_blueprint.parts.get("Turret").unwrap_or_else(|| chunk_blueprint.parts.values().next().unwrap_or(&default_part));
                    
                    let mut turret_atlas = None;
                    if let (Some(layout), false) = (&chunk_blueprint.atlas_layout, turret_part.frame_indices.is_empty()) {
                        turret_atlas = Some(TextureAtlas { layout: layout.clone(), index: turret_part.frame_indices[0] });
                    }

                    let turret_sprite = commands.spawn((
                         Sprite {
                             image: chunk_blueprint.texture.clone(),
                             custom_size: Some(Vec2::new(chunk_size, chunk_size)),
                             texture_atlas: turret_atlas,
                             ..default()
                         },
                         Transform::from_xyz(0.0, 0.0, 0.2), // Turret rotates visually on top of the base
                    )).id();

                    if !turret_part.frame_durations.is_empty() {
                       commands.entity(turret_sprite).insert(crate::textures::components::AnimationState { 
                           timer: Timer::new(std::time::Duration::from_millis(turret_part.frame_durations[0] as u64), TimerMode::Once),
                           current_index: 0,
                           frame_indices: turret_part.frame_indices.clone(),
                           frame_durations: turret_part.frame_durations.clone(),
                       });
                    }

                    turret_entity_opt = Some(turret_sprite);
                    commands.entity(base_sprite).add_child(turret_sprite);

                    Some(base_sprite)
                } else {
                    None
                }
            }
        );

        let base_cells: Vec<Entity> = grid.grid.iter().filter_map(|&e| e).collect();

        // Spawn a VoxelGrid wrapper holding the base cells
        let base_wrapper = commands.spawn((
            grid,
            Transform::from_xyz(0.0, 0.0, 1.0),
            Visibility::Inherited,
        )).id();
        
        commands.entity(base_wrapper).add_children(&base_cells);
        commands.entity(base_entity).add_child(base_wrapper);

        // Map behavior and rotation strictly to the isolated rotating turret part
        if let Some(turret_entity) = turret_entity_opt {
            commands.entity(turret_entity).insert(WeaponBundle {
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
            });
        }
        
        base.barrel_entity = Some(base_wrapper);
    }
}

pub fn turret_target_acquisition_system(
    mut turrets: Query<(Entity, &GlobalTransform, &Turret, &mut TargetControl)>,
    parents: Query<&ChildOf>,
    ships: Query<(&GlobalTransform, &ShipTargetingComputer), With<crate::ships::Ship>>,
    targets: Query<(&GlobalTransform, Option<&LinearVelocity>), (Without<TargetControl>, Without<Turret>)>,
) {
    for (turret_entity, turret_global, turret, mut target_control) in turrets.iter_mut() {
        // 1. Traverse up the hierarchy to find the parent ship
        let mut current = turret_entity;
        let mut root_ship = None;
        while let Ok(child_of) = parents.get(current) {
            current = child_of.get();
            root_ship = Some(current);
        }

        let Some(ship_entity) = root_ship else { continue; };
        let Ok((ship_global, targeting_comp)) = ships.get(ship_entity) else { continue; };

        let turret_pos = turret_global.translation().truncate();
        let ship_dir = (ship_global.compute_transform().rotation * Vec3::Y).truncate();
        let mount_forward_angle = ship_dir.y.atan2(ship_dir.x);

        // Candidate targets in priority order: Primary target first, then secondary targets
        let mut candidates = Vec::new();
        if let Some(primary) = targeting_comp.primary_target {
            candidates.push(primary);
        }
        for &secondary in &targeting_comp.secondary_targets {
            if Some(secondary) != targeting_comp.primary_target {
                candidates.push(secondary);
            }
        }

        let mut acquired_target = None;
        for candidate in candidates {
            if let Ok((target_global, _)) = targets.get(candidate) {
                let target_pos = target_global.translation().truncate();
                let is_primary = Some(candidate) == targeting_comp.primary_target;

                // Check firing arc
                let to_target = target_pos - turret_pos;
                if to_target.length_squared() > 0.001 {
                    let aim_angle = to_target.y.atan2(to_target.x);
                    if !crate::weapons::targeting::is_in_firing_arc(mount_forward_angle, aim_angle, turret.firing_arc) {
                        continue;
                    }
                }

                // Check range:
                // For player-commanded primary targets, allow tracking up to 1.5x max_range
                // so the turret aims at the target immediately while flying into range.
                // For opportunistic secondary targets, only acquire if within max_range.
                let max_track_range = if is_primary { turret.max_range * 1.5 } else { turret.max_range };
                if !crate::weapons::targeting::is_in_range(turret_pos, target_pos, max_track_range) {
                    continue;
                }

                acquired_target = Some(candidate);
                break;
            }
        }

        target_control.current_target = acquired_target;
    }
}

pub fn target_control_system(
    time: Res<Time>,
    mut turrets: Query<(&mut Transform, &GlobalTransform, &Turret, &mut TargetControl)>,
    targets: Query<(&GlobalTransform, Option<&LinearVelocity>), (Without<TargetControl>, Without<Turret>)>,
) {
    let dt = time.delta_secs();
    let predictor = FirstOrderPredictor;

    for (mut transform, global_transform, turret, mut target_control) in turrets.iter_mut() {
        if let Some(target_entity) = target_control.current_target {
            if let Ok((target_global, target_lin_vel)) = targets.get(target_entity) {
                let target_pos = target_global.translation().truncate();
                let target_vel = target_lin_vel.map(|v| v.0).unwrap_or(Vec2::ZERO);
                let current_pos = global_transform.translation().truncate();

                // Option A: First-Order lead prediction
                let aim_pos = predictor.predict_aim_point(current_pos, target_pos, target_vel, turret.projectile_speed);
                target_control.predicted_lead_pos = Some(aim_pos);

                let direction = aim_pos - current_pos;
                if direction.length_squared() < 0.001 {
                    continue;
                }

                let desired_world_angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;

                let current_world_rot = global_transform.compute_transform().rotation;
                let current_world_dir = current_world_rot * Vec3::Y;
                let current_world_angle = current_world_dir.y.atan2(current_world_dir.x) - std::f32::consts::FRAC_PI_2;

                let mut angle_diff = desired_world_angle - current_world_angle;
                while angle_diff > std::f32::consts::PI { angle_diff -= std::f32::consts::PI * 2.0; }
                while angle_diff < -std::f32::consts::PI { angle_diff += std::f32::consts::PI * 2.0; }

                let max_turn = turret.turn_speed * dt;
                
                let (axis, current_local_angle) = transform.rotation.to_axis_angle();
                let mut current_z_angle = if axis.z < 0.0 { -current_local_angle } else { current_local_angle };

                if angle_diff.abs() <= max_turn {
                    current_z_angle += angle_diff;
                    target_control.is_locked_on = true;
                } else {
                    current_z_angle += max_turn * angle_diff.signum();
                    target_control.is_locked_on = angle_diff.abs() < 0.15; 
                }

                transform.rotation = Quat::from_rotation_z(current_z_angle);
                target_control.current_aim_angle = current_z_angle;
            } else {
                target_control.current_target = None;
                target_control.is_locked_on = false;
                target_control.predicted_lead_pos = None;
            }
        } else {
            target_control.is_locked_on = false;
            target_control.predicted_lead_pos = None;
        }
    }
}

pub fn fire_control_system(
    mut commands: Commands,
    time: Res<Time>,
    // Fetch the GlobalTransform to accurately spawn the projectile in world coordinates
    mut weapons: Query<(Entity, &GlobalTransform, &Turret, &TargetControl, &mut FireControl)>,
    targets: Query<&GlobalTransform>,
    parents: Query<&ChildOf>,
) {
    let dt = time.delta();
    for (entity, global_transform, turret, target_control, mut fire_control) in weapons.iter_mut() {
        fire_control.reload_timer.tick(dt);

        // Only fire if the target is within the turret's max weapon firing range
        let in_range = if let Some(target_entity) = target_control.current_target {
            if let Ok(target_global) = targets.get(target_entity) {
                let target_pos = target_global.translation().truncate();
                let turret_pos = global_transform.translation().truncate();
                crate::weapons::targeting::is_in_range(turret_pos, target_pos, turret.max_range)
            } else {
                false
            }
        } else {
            false
        };

        if target_control.is_locked_on && in_range && fire_control.reload_timer.just_finished() {
            let transform = global_transform.compute_transform();
            
            // Assume 0 rotation natively points UP (Vec3::Y), so we compute local Up mapped to world:
            let direction = transform.rotation * Vec3::Y;
            let velocity = direction.truncate() * turret.projectile_speed;

            if turret.weapon_type == "plasma" {
                // Traverse up to find the root parent (which is the ship)
                let mut current_entity = entity;
                let mut source_ship = None;
                while let Ok(parent) = parents.get(current_entity) {
                    current_entity = parent.get();
                    source_ship = Some(current_entity);
                }

                commands.spawn(PlasmaProjectileBundle {
                    plasma_projectile_tag: PlasmaProjectile,
                    plasma_projectile_stats: PlasmaProjectileStats {
                        damage: turret.damage as i32,
                        energy: 100,
                        energy_loss_rate: 10,
                        source_ship,
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
                    collision_layers: CollisionLayers::new(
                        [crate::physics::GameLayer::Projectile],
                        [crate::physics::GameLayer::Ship, crate::physics::GameLayer::Shield, crate::physics::GameLayer::Default]
                    ),
                    sensor: Sensor,
                    velocity: LinearVelocity(velocity),
                    colliding_entities: CollidingEntities::default(),
                });
            } else {
                info!("Unknown weapon_type: {}", turret.weapon_type);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ships::Ship;

    #[test]
    fn test_turret_pipeline_acquires_and_fires() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(TransformPlugin);
        app.add_systems(Update, (
            turret_target_acquisition_system,
            target_control_system,
            fire_control_system,
        ).chain());

        // Spawn target at (0, 150) - exactly like DummyTarget in main.rs
        let target = app.world_mut().spawn((
            Transform::from_xyz(0.0, 150.0, 0.0),
            GlobalTransform::default(),
        )).id();

        // Spawn ship at (-200, 0) - exactly like Ship 1 in main.rs
        let mut comp = ShipTargetingComputer::default();
        comp.set_primary(Some(target));
        let ship = app.world_mut().spawn((
            Ship,
            comp,
            Transform::from_xyz(-200.0, 0.0, 0.0),
            GlobalTransform::default(),
        )).id();

        // Spawn turret as child of ship
        let turret = app.world_mut().spawn((
            Turret {
                weapon_type: "plasma".to_string(),
                damage: 10.0,
                max_range: 500.0,
                firing_arc: std::f32::consts::TAU,
                turn_speed: 100000.0,
                projectile_speed: 400.0,
            },
            TargetControl::new(),
            FireControl::new(10000.0), // Finishes in 0.1ms
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
            ChildOf(ship),
        )).id();

        // Advance time by 0.5 seconds to let the turret turn and timer fire
        for _ in 0..50 {
            app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(20));
            app.update();
        }

        let control = app.world().get::<TargetControl>(turret).unwrap();
        println!("After 1s: is_locked_on = {}", control.is_locked_on);
        assert!(control.is_locked_on);

        // Check if a PlasmaProjectile was spawned
        let projectile_count = app.world_mut().query::<&crate::projectiles::PlasmaProjectile>().iter(app.world()).count();
        println!("Spawned projectiles: {}", projectile_count);
        assert!(projectile_count > 0, "Turret should have spawned at least 1 projectile!");
    }
}

