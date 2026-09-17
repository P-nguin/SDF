// In src/ships/plugins/effects_plugin.rs
use bevy::prelude::*;

use crate::cells::systems::*;

pub struct CellEffectPlugin;

impl Plugin for CellEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, shield_builder_system);
    }
}