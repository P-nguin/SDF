use bevy::prelude::*;

use crate::cells::components::*;

pub fn load_cell_registry(
    mut commands: Commands,
) {
    let mut registry = CellRegistry::default();

    registry.cells.insert("-".to_string(), Cell::new(
        "-".to_string(),
        100.0,
        100.0,
        0.5,
        0.0,
        0.0,
        0.0,
        0.0,
    ));

    registry.cells.insert("+".to_string(), Cell::new(
        "+".to_string(),
        100.0,
        100.0,
        0.0,
        50.0,
        0.0,
        0.0,
        0.0,
    ));

    registry.cells.insert("*".to_string(), Cell::new(
        "*".to_string(),
        100.0,
        100.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
    ));
    
    registry.cells.insert("(".to_string(), Cell::new(
        "(".to_string(),
        100.0,
        100.0,
        0.0,
        0.0,
        0.0,
        100.0,
        100.0,
    ));

    registry.cells.insert("sb".to_string(), Cell::new(
        "sb".to_string(),
        100.0,
        100.0,
        0.0,
        0.0,
        0.0,
        100.0,
        100.0,
    ));

    registry.cells.insert("T".to_string(), Cell::new(
        "T".to_string(),
        100.0,
        100.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
    ));

    commands.insert_resource(registry);
}