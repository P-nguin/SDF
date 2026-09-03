use bevy::prelude::*;
use crate::weapons::*;

pub fn aim_system(
    time: Res<Time>,
    turrets: Query<&Turret>,
    mut bases: Query<(&TurretBase, &mut Transform)>,
    // Use Without<TurretBase> to assure Bevy these queries do not overlap in memory
    mut barrels: Query<(&TurretBarrel, &mut Transform), Without<TurretBase>>,
    global_transforms: Query<&GlobalTransform>,
) {
    for turret in turrets.iter() {
        let Some(target_entity) = turret.target else { continue; };
        let Ok(target_global) = global_transforms.get(target_entity) else { continue; };
        let target_pos = target_global.translation();

        // 1. Aim the Base (Yaw / Horizontal)
        if let (Ok(base_global), Ok((base, mut base_transform))) = (
            global_transforms.get(turret.base_entity),
            bases.get_mut(turret.base_entity)
        ) {
            // Convert target to base's local coordinate space
            let local_target = base_global.affine().inverse().transform_point3(target_pos);
            
            // Calculate horizontal angle to target.
            // Your custom SDF cylinder was built facing +Z, so we calculate atan2 against +Z
            let delta_yaw = local_target.x.atan2(local_target.z);
            
            let max_turn = base.turn_speed * time.delta_secs();
            base_transform.rotate_local_y(delta_yaw.clamp(-max_turn, max_turn));
        }

        // 2. Aim the Barrel (Pitch / Vertical)
        if let (Ok(barrel_global), Ok((barrel, mut barrel_transform))) = (
            global_transforms.get(turret.barrel_entity),
            barrels.get_mut(turret.barrel_entity)
        ) {
            // Convert target to barrel's local coordinate space
            let local_target = barrel_global.affine().inverse().transform_point3(target_pos);
            
            // Calculate vertical angle. 
            // Positive rotation around local X pitches down towards +Z. We negate it to pitch up towards +Y.
            let delta_pitch = -local_target.y.atan2(local_target.z);
            
            let max_pitch = barrel.pitch_speed * time.delta_secs();
            barrel_transform.rotate_local_x(delta_pitch.clamp(-max_pitch, max_pitch));

            // Clamp absolute elevation/depression limits
            let (yaw, mut pitch, roll) = barrel_transform.rotation.to_euler(EulerRot::YXZ);
            pitch = pitch.clamp(barrel.min_depression, barrel.max_elevation);
            barrel_transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
        }
    }
}