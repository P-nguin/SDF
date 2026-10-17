use bevy::prelude::*;
use avian2d::prelude::*;
use crate::navigation::components::*;
use crate::ships::Ship;

pub fn debug_navigation_system(
    mut gizmos: Gizmos,
    query: Query<(
        &Transform,
        &LinearVelocity,
        &NavigationPlan,
    ), With<Ship>>,
) {
    for (transform, lin_vel, plan) in query.iter() {
        let current_pos = transform.translation.truncate();
        
        // Draw velocity vector (Blue)
        if lin_vel.0.length_squared() > 1.0 {
            gizmos.arrow_2d(current_pos, current_pos + lin_vel.0, bevy::color::palettes::css::BLUE);
        }

        // Draw Waypoints
        let mut prev_pos = current_pos;
        for waypoint in plan.waypoints.iter() {
            // Draw path line
            gizmos.line_2d(prev_pos, waypoint.position, bevy::color::palettes::css::WHITE);
            
            // Draw waypoint radius
            gizmos.circle_2d(waypoint.position, waypoint.radius, bevy::color::palettes::css::GREEN);

            // Draw target heading at the waypoint (Red)
            if waypoint.target_heading.length_squared() > 0.1 {
                gizmos.arrow_2d(
                    waypoint.position, 
                    waypoint.position + waypoint.target_heading * 20.0, 
                    bevy::color::palettes::css::RED
                );
            }
            
            prev_pos = waypoint.position;
        }
    }
}
