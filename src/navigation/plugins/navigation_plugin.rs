use bevy::prelude::*;
use crate::navigation::systems::*;

pub struct NavigationPlugin;

impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            thruster_execution_system,
            debug_navigation_system,
        ));
    }
}