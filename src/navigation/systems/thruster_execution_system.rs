use bevy::prelude::*;
use avian2d::prelude::*;
use crate::navigation::components::*;
use crate::ships::Ship;

pub fn thruster_execution_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut NavigationPlan,
        &MovementProfile,
        &Transform,
        &mut LinearVelocity,
        &mut AngularVelocity,
    ), With<Ship>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut plan, profile, transform, mut lin_vel, mut ang_vel) in query.iter_mut() {
        // 1. EMPTY PLAN: Rapidly bleed off speed (artificial braking)
        if plan.waypoints.is_empty() {
            // Decelerate at 3x max speed per second (adjust this for feel)
            let decel_step = profile.max_speed * 3.0 * dt; 
            
            if lin_vel.0.length() <= decel_step {
                lin_vel.0 = Vec2::ZERO;
            } else {
                let v_dir = lin_vel.0.normalize();
                lin_vel.0 -= v_dir * decel_step;
            }

            // Also damp rotation rapidly
            let rot_step = profile.max_turn_rate * 3.0 * dt;
            if ang_vel.0.abs() <= rot_step {
                ang_vel.0 = 0.0;
            } else {
                ang_vel.0 -= ang_vel.0.signum() * rot_step;
            }

            // Once completely stopped, remove the plan
            if lin_vel.0 == Vec2::ZERO && ang_vel.0 == 0.0 {
                commands.entity(entity).remove::<NavigationPlan>();
            }
            continue;
        }

        let current_pos = transform.translation.truncate();
        let waypoint = plan.waypoints.front().unwrap().clone();
        let distance = (waypoint.position - current_pos).length();
        let direction = if distance > 0.001 { (waypoint.position - current_pos) / distance } else { Vec2::ZERO };

        // 1. Arrival Check
        let moving_away = if lin_vel.0.length_squared() > 1.0 {
            lin_vel.0.normalize().dot(direction) < 0.0
        } else {
            false
        };
                 
        // RELAXED WAYPOINT: The faster we are moving, the larger our acceptable "miss" radius becomes.
        // We add half our current speed to the base radius to create a dynamic hitbox.
        let dynamic_tolerance = waypoint.radius.max(waypoint.radius * 2.0 + (lin_vel.0.length() * 0.5));
        let close_miss = distance <= dynamic_tolerance && moving_away;

        // Arrive if we are inside the exact radius, or if we passed through our dynamic tolerance bubble
        if distance <= waypoint.radius || close_miss {
            plan.waypoints.pop_front();
            
            // If the plan is empty, ONLY remove it immediately if we were supposed to keep moving.
            // If target_speed == 0.0, we leave the component on the entity so the braking block above runs next frame.
            if plan.waypoints.is_empty() && waypoint.target_speed > 0.0 {
                commands.entity(entity).remove::<NavigationPlan>();
            }
            continue;
        }

        // 2. Speed Control
        let slowing_radius = (profile.max_speed * profile.max_speed) / (2.0 * profile.max_reverse_thrust);

        // FIX 1: Use square root curve for constant deceleration
        let target_speed = if distance < slowing_radius {
            (2.0 * profile.max_reverse_thrust * distance).sqrt().min(profile.max_speed)
        } else {
            profile.max_speed
        };
        
        let desired_velocity = direction * target_speed;
        
        // Subtract current velocity from desired velocity to cancel momentum
        let steering_vec = desired_velocity - lin_vel.0;
        let steering_mag = steering_vec.length();

        // 3. Heading Control
        let forward = (transform.rotation * Vec3::Y).truncate().normalize();
        
        // FIX 2: Do not snap to the target_heading if we still have significant momentum to shed.
        // If the ship rotates while braking, it loses its reverse thrusters and relies on weaker strafe thrusters, causing overshoots.
        let desired_heading = if distance > waypoint.radius * 2.0 || lin_vel.0.length_squared() > 400.0 {
            direction // Face travel direction while moving fast
        } else {
            waypoint.target_heading // Align to final heading only when slow enough
        };
        
        let mut angle_diff = forward.angle_to(desired_heading);
        
        if desired_heading.length_squared() < 0.01 {
             angle_diff = 0.0;
        }

        if angle_diff.abs() > 0.01 {
            ang_vel.0 = (angle_diff * 5.0).clamp(-profile.max_turn_rate, profile.max_turn_rate);
        } else {
            ang_vel.0 = 0.0;
        }

        // 4. Apply Thrust
        if steering_mag > 0.1 {
            let right = (transform.rotation * Vec3::X).truncate().normalize();

            // Project the required steering force onto the ship's local axes
            let local_forward_req = steering_vec.dot(forward);
            let local_right_req = steering_vec.dot(right);

            // Clamp the requested forces to the physical limits of the ship's thrusters
            let applied_forward = if local_forward_req > 0.0 {
                local_forward_req.min(profile.max_forward_thrust * dt)
            } else {
                local_forward_req.max(-profile.max_reverse_thrust * dt)
            };

            let applied_right = local_right_req.clamp(-profile.max_strafe_thrust * dt, profile.max_strafe_thrust * dt);

            // Apply to linear velocity (Omnidirectional thrust)
            lin_vel.0 += forward * applied_forward + right * applied_right;
        }
    }
}
