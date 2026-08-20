use bevy::prelude::*;

use crate::camera::{CameraState, CameraStateTag};
use crate::camera::Selected;

pub fn focus_on_selected_ship(
    keyboard: Res<ButtonInput<KeyCode>>,
    selected_ships: Query<Entity, With<Selected>>,
    mut camera_state_query: Query<&mut CameraState, With<CameraStateTag>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyF) { return; }

    let Ok(selected_entity) = selected_ships.single() else { return; };
    let Ok(mut camera_state) = camera_state_query.single_mut() else { return; };

    camera_state.focus_target = Some(selected_entity);
}
