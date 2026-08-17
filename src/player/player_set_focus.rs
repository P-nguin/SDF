use bevy::prelude::*;

use crate::camera::PanOrbitState;
use crate::player::Selected;

pub fn focus_on_selected_ship(
    keyboard: Res<ButtonInput<KeyCode>>,
    selected_ships: Query<Entity, With<Selected>>,
    mut cameras: Query<&mut PanOrbitState>,
) {
    // Only run if F was just pressed
    if !keyboard.just_pressed(KeyCode::KeyF) {
        return;
    }

    // Check if there is currently a selected ship
    let Ok(selected_entity) = selected_ships.single() else {
        return;
    };

    // Tell the camera to focus on it
    for mut state in &mut cameras {
        state.focus = Some(selected_entity);
    }
}
