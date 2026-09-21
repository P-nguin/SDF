use bevy::prelude::*;
use crate::weapons::systems::*;

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            turret_builder_system,
            target_control_system,
            fire_control_system,
        ).chain());
    }
}
