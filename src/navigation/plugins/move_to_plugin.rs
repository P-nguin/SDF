use bevy::prelude::*;
use crate::navigation::move_to_system;

pub struct MoveToPlugin;

impl Plugin for MoveToPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_to_system);
    }
}