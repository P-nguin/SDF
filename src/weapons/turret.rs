use bevy::prelude::*;

#[derive(Component)]
pub struct TurretBase {
    pub health: f32,
    pub turn_speed: f32,
}

#[derive(Component)]
pub struct TurretBarrel {
    pub health: f32,
    pub pitch_speed: f32,
    pub min_depression: f32,
    pub max_elevation: f32,
}

#[derive(Component)]
pub struct Turret {
    pub base_entity: Entity, 
    pub barrel_entity: Entity,

    pub health: f32,
    pub damage: f32,

    pub reload_time:f32,
    pub reload:f32,

    pub range:f32,

    pub target: Option<Entity>,
}

