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

#[derive(Component, Debug, Default, Clone)]
pub struct ShipTargetingComputer {
    pub primary_target: Option<Entity>,
    pub secondary_targets: Vec<Entity>,
}

impl ShipTargetingComputer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_primary(&mut self, target: Option<Entity>) {
        self.primary_target = target;
    }

    pub fn add_secondary(&mut self, target: Entity) {
        if !self.secondary_targets.contains(&target) {
            self.secondary_targets.push(target);
        }
    }

    pub fn remove_target(&mut self, target: Entity) {
        if self.primary_target == Some(target) {
            self.primary_target = None;
        }
        self.secondary_targets.retain(|&e| e != target);
    }
}

#[derive(Component, Clone)]
pub struct TargetControl {
    pub current_target: Option<Entity>,
    pub current_aim_angle: f32,
    pub is_locked_on: bool,
    pub predicted_lead_pos: Option<Vec2>,
}
impl TargetControl {
    pub fn new() -> Self {
        TargetControl {
            current_target: None,
            current_aim_angle: 0.0,
            is_locked_on: false,
            predicted_lead_pos: None,
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
