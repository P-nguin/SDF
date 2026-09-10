// In src/navigation/systems/move_to_system.rs
use avian2d::prelude::*;
use bevy::prelude::*;
use crate::navigation::MoveTo;
use crate::ships::{Ship, ShipNav};

pub fn move_to_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &MoveTo,
        &ShipNav,
        &Transform,
        &mut LinearVelocity,
        &mut AngularVelocity,
    ), With<Ship>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, move_to, nav, transform, mut lin_vel, mut ang_vel) in query.iter_mut() {
        let current_pos = transform.translation.truncate();
        let to_target = move_to.target - current_pos;
        let distance = to_target.length();

        // 1. Arrival Check
        if distance <= move_to.rad {
            commands.entity(entity).remove::<MoveTo>();
            lin_vel.0 = Vec2::ZERO;
            ang_vel.0 = 0.0;
            continue;
        }

        let desired_dir = to_target / distance;

        // 2. Determine Forward Vector (assuming local +Y is forward)
        let forward = (transform.rotation * Vec3::Y).truncate().normalize();

        // 3. Rotation Steering (signed angle from forward to target direction)
        let angle_diff = forward.angle_to(desired_dir);

        if angle_diff.abs() > 0.05 {
            // Rotate towards the target clamped by max_turn
            ang_vel.0 = (angle_diff * 5.0).clamp(-nav.max_turn, nav.max_turn);
        } else {
            ang_vel.0 = 0.0;
        }

        // 4. Linear Propulsion
        // Only apply forward thrust if roughly pointing toward the destination
        if angle_diff.abs() < std::f32::consts::FRAC_PI_3 {
            let target_vel = forward * nav.max_speed;
            lin_vel.0 = lin_vel.0.move_towards(target_vel, nav.max_accel * dt);
        } else {
            // Decelerate while performing sharp turns
            lin_vel.0 = lin_vel.0.move_towards(Vec2::ZERO, nav.max_accel * dt);
        }
    }
}