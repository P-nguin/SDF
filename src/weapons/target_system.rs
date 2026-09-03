// In src/weapons/target_system.rs
use bevy::prelude::*;
use bevy::ecs::relationship::Relationship; // <-- Added this trait
use crate::weapons::*;
use crate::ship::ShipTag;

pub fn target_system(
    mut turrets: Query<&mut Turret>,
    global_transforms: Query<&GlobalTransform>,
    parents: Query<&ChildOf>,
    ships: Query<(Entity, &GlobalTransform), With<ShipTag>>,
) {
    for mut turret in turrets.iter_mut() {
        // 1. Get the turret's true world position via its physical base
        let Ok(base_transform) = global_transforms.get(turret.base_entity) else { continue; };
        let turret_pos = base_transform.translation();

        // 2. Identify the ship hull this turret is attached to
        let own_ship = parents.get(turret.base_entity).map(|p| p.get()).ok();

        let mut closest_enemy = None;
        let mut closest_dist = f32::MAX;

        // 3. Find the closest ship that isn't ours
        for (ship_entity, ship_transform) in ships.iter() {
            if Some(ship_entity) == own_ship {
                continue; // Do not target ourselves
            }

            let dist = turret_pos.distance(ship_transform.translation());
            
            // Only lock on if the enemy is the closest so far AND within weapon range
            if dist < closest_dist && dist <= turret.range {
                closest_dist = dist;
                closest_enemy = Some(ship_entity);
            }
        }

        // 4. Update the target lock
        turret.target = closest_enemy;
    }
}