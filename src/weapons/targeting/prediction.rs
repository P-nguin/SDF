use bevy::prelude::*;

/// Trait defining a strategy for predicting target aim points.
pub trait TargetPredictor: Send + Sync {
    fn predict_aim_point(
        &self,
        shooter_pos: Vec2,
        target_pos: Vec2,
        target_vel: Vec2,
        projectile_speed: f32,
    ) -> Vec2;
}

/// Option A: First-order distance-over-speed approximation.
/// Calculates estimated flight time t = distance / projectile_speed,
/// and aims at target_pos + target_vel * t.
#[derive(Default, Debug, Clone, Copy)]
pub struct FirstOrderPredictor;

impl TargetPredictor for FirstOrderPredictor {
    fn predict_aim_point(
        &self,
        shooter_pos: Vec2,
        target_pos: Vec2,
        target_vel: Vec2,
        projectile_speed: f32,
    ) -> Vec2 {
        if projectile_speed <= 0.0 {
            return target_pos;
        }

        let distance = shooter_pos.distance(target_pos);
        let flight_time = distance / projectile_speed;
        target_pos + target_vel * flight_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stationary_target() {
        let predictor = FirstOrderPredictor;
        let shooter = Vec2::ZERO;
        let target = Vec2::new(100.0, 0.0);
        let vel = Vec2::ZERO;
        let speed = 200.0;

        let aim_pos = predictor.predict_aim_point(shooter, target, vel, speed);
        assert_eq!(aim_pos, target);
    }

    #[test]
    fn test_moving_target_lead() {
        let predictor = FirstOrderPredictor;
        let shooter = Vec2::ZERO;
        let target = Vec2::new(100.0, 0.0);
        let vel = Vec2::new(0.0, 50.0); // Moving upward at 50 units/s
        let speed = 100.0; // Travel time = 100 / 100 = 1.0s

        let aim_pos = predictor.predict_aim_point(shooter, target, vel, speed);
        // Should lead 50 units up
        assert_eq!(aim_pos, Vec2::new(100.0, 50.0));
    }

    #[test]
    fn test_zero_speed_fallback() {
        let predictor = FirstOrderPredictor;
        let shooter = Vec2::ZERO;
        let target = Vec2::new(100.0, 0.0);
        let vel = Vec2::new(0.0, 50.0);
        let speed = 0.0;

        let aim_pos = predictor.predict_aim_point(shooter, target, vel, speed);
        assert_eq!(aim_pos, target);
    }
}
