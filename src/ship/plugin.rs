use bevy::prelude::*;
use crate::ship::toggle_ship_render_mode;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_ship_render_mode);
    }
}