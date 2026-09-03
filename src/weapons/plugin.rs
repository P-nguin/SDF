use bevy::prelude::*;
use crate::weapons::*;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (target_system, reload_system, aim_system, fire_system));
    }
}