use bevy::prelude::*;

use crate::cells::load_cell_registry;

pub struct CellRegistryPlugin;

impl Plugin for CellRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_cell_registry);
    }
}