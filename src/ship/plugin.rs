use bevy::prelude::*;
use crate::ship::*;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, process_nav_commands);
    }
}