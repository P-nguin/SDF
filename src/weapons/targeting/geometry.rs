use bevy::prelude::*;

/// Checks if target is within maximum weapon engagement range.
pub fn is_in_range(shooter_pos: Vec2, target_pos: Vec2, max_range: f32) -> bool {
    shooter_pos.distance_squared(target_pos) <= max_range * max_range
}

/// Normalizes an angle into (-PI, PI].
pub fn normalize_angle(mut angle: f32) -> f32 {
    while angle > std::f32::consts::PI {
        angle -= std::f32::consts::TAU;
    }
    while angle <= -std::f32::consts::PI {
        angle += std::f32::consts::TAU;
    }
    angle
}

/// Checks if a target angle falls within the turret's mounting arc.
/// `mount_forward_angle` is the orientation of the turret mount / ship in world space.
/// `aim_angle` is the world-space angle from turret to target.
/// `firing_arc` is the total angular span in radians (e.g. PI is a 180° forward arc, TAU is 360°).
pub fn is_in_firing_arc(
    mount_forward_angle: f32,
    aim_angle: f32,
    firing_arc: f32,
) -> bool {
    if firing_arc >= std::f32::consts::TAU - 0.001 {
        return true; // Full 360 degree coverage
    }
    let diff = normalize_angle(aim_angle - mount_forward_angle);
    diff.abs() <= (firing_arc / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_in_range() {
        let shooter = Vec2::ZERO;
        assert!(is_in_range(shooter, Vec2::new(50.0, 0.0), 100.0));
        assert!(is_in_range(shooter, Vec2::new(100.0, 0.0), 100.0));
        assert!(!is_in_range(shooter, Vec2::new(100.1, 0.0), 100.0));
    }

    #[test]
    fn test_is_in_firing_arc() {
        // Facing Up (PI/2), 90 degree total arc (PI/2) -> allowed range: [PI/4, 3PI/4]
        let forward = std::f32::consts::FRAC_PI_2;
        let arc = std::f32::consts::FRAC_PI_2;

        // Straight ahead
        assert!(is_in_firing_arc(forward, std::f32::consts::FRAC_PI_2, arc));
        // 45 deg right (within half-arc)
        assert!(is_in_firing_arc(forward, std::f32::consts::FRAC_PI_4, arc));
        // Directly behind
        assert!(!is_in_firing_arc(forward, -std::f32::consts::FRAC_PI_2, arc));
    }
}
