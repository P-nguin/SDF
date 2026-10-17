use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct MovementProfile {
    pub max_forward_thrust: f32,
    pub max_reverse_thrust: f32,
    pub max_strafe_thrust: f32,
    pub max_turn_rate: f32,
    pub max_speed: f32,
}
