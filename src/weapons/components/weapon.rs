use bevy::prelude::*;

#[derive(Component)]
pub struct TurretBase {
    pub weapon_type: String,
    pub damage: f32,
    pub max_range: f32,
    pub firing_arc: f32, // in radians
    pub turn_speed: f32, // in radians per second
    pub projectile_speed: f32,
    pub fire_rate: f32,

    pub barrel_entity: Option<Entity>,
}


#[derive(Component, Clone)]
pub struct Turret {
    pub weapon_type: String,
    pub damage: f32,
    pub max_range: f32,
    pub firing_arc: f32, // in radians
    pub turn_speed: f32, // in radians per second
    pub projectile_speed: f32,
}

#[derive(Component)]
pub struct TargetControl {
    pub current_target: Option<Entity>,
    pub current_aim_angle: f32,
    pub is_locked_on: bool,
}
impl TargetControl {
    pub fn new() -> Self {
        TargetControl {
            current_target: None,
            current_aim_angle: 0.0,
            is_locked_on: false,
        }
    }
}

#[derive(Component)]
pub struct FireControl {
    pub reload_timer: Timer,
}
impl FireControl {
    pub fn new(fire_rate: f32) -> Self {
        let duration = if fire_rate > 0.0 { 1.0 / fire_rate } else { 1.0 };
        FireControl {
            reload_timer: Timer::from_seconds(duration, TimerMode::Repeating),
        }
    }
}

#[derive(Bundle)]
pub struct WeaponBundle {
    pub turret: Turret,
    pub target_control: TargetControl,
    pub fire_control: FireControl,
}
