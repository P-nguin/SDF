use bevy::prelude::*;
use crate::weapons::*;

pub fn fire_system(
    mut turrets_query: Query<(Entity, &mut Turret)>, // Removed &GlobalTransform
    turrets_base_query: Query<&TurretBase>,
    turrets_barrel_query: Query<&TurretBarrel>,
    global_transforms: Query<&GlobalTransform>, // Added to get positions
) {
    for (turret_entity, mut turret) in turrets_query.iter_mut() {
        if turret.reload > 0.0 { continue; }

        let (Ok(base), Ok(barrel)) = (
            turrets_base_query.get(turret.base_entity), 
            turrets_barrel_query.get(turret.barrel_entity)
        ) else { continue; };
        
        if turret.health <= 0.0 || base.health <= 0.0 || barrel.health <= 0.0 { continue; }

        let Some(target_entity) = turret.target else { continue; };
        
        // Fetch target position AND the turret's base position
        if let (Ok(target_transform), Ok(base_transform)) = (
            global_transforms.get(target_entity),
            global_transforms.get(turret.base_entity)
        ) {
            let distance = base_transform.translation().distance(target_transform.translation());
            
            if distance <= turret.range {
                info!("Turret {:?} fired at {:?}!", turret_entity, target_entity);
                turret.reload = turret.reload_time;
                // commands.spawn(( ...ProjectileBundle... ));
            }
        } else {
            turret.target = None;
        }
    }
}