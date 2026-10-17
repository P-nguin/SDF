use bevy::prelude::*;
use crate::controls::components::*;
use crate::ships::Ship;
use crate::weapons::components::{ShipTargetingComputer, TargetControl};

pub fn handle_command_mode_input(
    mut command_mode: ResMut<CommandMode>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    q_selected_ships: Query<Entity, (With<Ship>, With<Selected>)>,
    mut q_targeting_computers: Query<&mut ShipTargetingComputer>,
    q_targetables: Query<(Entity, &GlobalTransform), With<Targetable>>,
) {
    // 1. Press 'T' to toggle targeting mode for currently selected ships
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        if !q_selected_ships.is_empty() {
            if *command_mode == CommandMode::Targeting {
                *command_mode = CommandMode::Normal;
            } else {
                *command_mode = CommandMode::Targeting;
            }
        }
    }

    // 2. If in targeting mode, process player actions
    if *command_mode == CommandMode::Targeting {
        // Cancel if Escape or Right Click
        if keyboard_input.just_pressed(KeyCode::Escape) || mouse_input.just_pressed(MouseButton::Right) {
            *command_mode = CommandMode::Normal;
            return;
        }

        // On Left Click, assign or clear target
        if mouse_input.just_pressed(MouseButton::Left) {
            let Ok(window) = q_window.single() else { return; };
            let Ok((camera, camera_transform)) = q_camera.single() else { return; };
            let Some(cursor_pos) = window.cursor_position() else { return; };
            let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return; };

            let click_radius = 80.0;
            let mut closest_target: Option<(Entity, f32)> = None;

            for (target_entity, target_global) in q_targetables.iter() {
                let pos = target_global.translation().truncate();
                let dist = pos.distance(world_pos);
                if dist <= click_radius {
                    if closest_target.is_none() || dist < closest_target.unwrap().1 {
                        closest_target = Some((target_entity, dist));
                    }
                }
            }

            if let Some((target_entity, _)) = closest_target {
                for ship_entity in q_selected_ships.iter() {
                    // Ship cannot target itself
                    if ship_entity != target_entity {
                        if let Ok(mut comp) = q_targeting_computers.get_mut(ship_entity) {
                            comp.set_primary(Some(target_entity));
                        }
                    }
                }
            } else {
                // If clicked open space, clear primary target
                for ship_entity in q_selected_ships.iter() {
                    if let Ok(mut comp) = q_targeting_computers.get_mut(ship_entity) {
                        comp.set_primary(None);
                    }
                }
            }

            *command_mode = CommandMode::Normal;
        }
    }
}

pub fn handle_selection_input(
    mut commands: Commands,
    mut selection_state: ResMut<SelectionState>,
    command_mode: Res<CommandMode>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    q_ships: Query<(Entity, &GlobalTransform), With<Ship>>,
    q_selected: Query<Entity, With<Selected>>,
) {
    // If in targeting mode, ignore selection input
    if *command_mode != CommandMode::Normal {
        selection_state.is_dragging = false;
        return;
    }

    let Ok(window) = q_window.single() else { return; };
    let Ok((camera, camera_transform)) = q_camera.single() else { return; };

    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return; };

    if mouse_input.just_pressed(MouseButton::Left) {
        selection_state.is_dragging = true;
        selection_state.start_pos = world_pos;
        selection_state.current_pos = world_pos;
    } else if mouse_input.pressed(MouseButton::Left) {
        selection_state.current_pos = world_pos;
    } else if mouse_input.just_released(MouseButton::Left) {
        selection_state.is_dragging = false;

        let min_x = selection_state.start_pos.x.min(selection_state.current_pos.x);
        let max_x = selection_state.start_pos.x.max(selection_state.current_pos.x);
        let min_y = selection_state.start_pos.y.min(selection_state.current_pos.y);
        let max_y = selection_state.start_pos.y.max(selection_state.current_pos.y);

        let is_click = (max_x - min_x) < 5.0 && (max_y - min_y) < 5.0;

        if !keyboard_input.pressed(KeyCode::ShiftLeft) && !keyboard_input.pressed(KeyCode::ShiftRight) {
            for entity in q_selected.iter() {
                commands.entity(entity).remove::<Selected>();
            }
        }

        for (entity, transform) in q_ships.iter() {
            let pos = transform.translation().truncate();
            
            let is_inside = if is_click {
                pos.distance(world_pos) < 100.0 
            } else {
                pos.x >= min_x && pos.x <= max_x && pos.y >= min_y && pos.y <= max_y
            };

            if is_inside {
                commands.entity(entity).insert(Selected);
            }
        }
    }
}

pub fn render_selection_box_system(
    mut gizmos: Gizmos,
    selection_state: Res<SelectionState>,
) {
    if selection_state.is_dragging {
        let center = (selection_state.start_pos + selection_state.current_pos) / 2.0;
        let size = (selection_state.current_pos - selection_state.start_pos).abs();
        gizmos.rect_2d(center, size, Color::srgba(0.0, 1.0, 0.0, 0.5));
    }
}

pub fn render_selected_units_system(
    mut gizmos: Gizmos,
    q_selected: Query<&GlobalTransform, (With<Ship>, With<Selected>)>,
) {
    for transform in q_selected.iter() {
        let pos = transform.translation().truncate();
        gizmos.circle_2d(pos, 60.0, Color::srgba(0.0, 1.0, 0.0, 0.8));
    }
}

pub fn render_targeting_ui_system(
    mut gizmos: Gizmos,
    command_mode: Res<CommandMode>,
    q_window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    q_targetables: Query<(Entity, &GlobalTransform), With<Targetable>>,
    q_selected_ships: Query<&ShipTargetingComputer, (With<Ship>, With<Selected>)>,
    q_turrets: Query<&TargetControl>,
) {
    let Ok(window) = q_window.single() else { return; };
    let Ok((camera, camera_transform)) = q_camera.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return; };

    // 1. In targeting mode: draw crosshair and hover highlight
    if *command_mode == CommandMode::Targeting {
        // Red crosshair reticle at cursor
        gizmos.circle_2d(world_pos, 20.0, Color::srgb(1.0, 0.2, 0.2));
        gizmos.line_2d(world_pos - Vec2::new(28.0, 0.0), world_pos + Vec2::new(28.0, 0.0), Color::srgb(1.0, 0.2, 0.2));
        gizmos.line_2d(world_pos - Vec2::new(0.0, 28.0), world_pos + Vec2::new(0.0, 28.0), Color::srgb(1.0, 0.2, 0.2));

        // Highlight hovered targetable
        for (_entity, global) in q_targetables.iter() {
            let pos = global.translation().truncate();
            if pos.distance(world_pos) <= 80.0 {
                gizmos.circle_2d(pos, 50.0, Color::srgba(1.0, 0.2, 0.2, 0.9));
            }
        }
    }

    // 2. Draw red target lock indicator around primary targets of selected ships
    for comp in q_selected_ships.iter() {
        if let Some(target_entity) = comp.primary_target {
            if let Ok((_, target_global)) = q_targetables.get(target_entity) {
                let pos = target_global.translation().truncate();
                gizmos.circle_2d(pos, 45.0, Color::srgba(0.9, 0.1, 0.1, 0.7));
            }
        }
    }

    // 3. Draw predicted lead point indicator for locked turrets (yellow/orange dot)
    for control in q_turrets.iter() {
        if let Some(lead_pos) = control.predicted_lead_pos {
            gizmos.circle_2d(lead_pos, 4.0, Color::srgb(1.0, 0.8, 0.0));
        }
    }
}
