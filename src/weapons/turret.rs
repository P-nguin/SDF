use bevy::prelude::*;

#[derive(Component)]
pub struct TurretBase {
    pub turn_speed: f32,
}

#[derive(Component)]
pub struct TurretBarrel {
    pub pitch_speed: f32,
    pub min_depression: f32,
    pub max_elevation: f32,
}