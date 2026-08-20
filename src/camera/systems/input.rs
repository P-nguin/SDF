use bevy::prelude::*;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use crate::camera::components::*;

pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut evr_scroll: MessageReader<MouseWheel>, 
    mut camera_state_query: Query<&mut CameraState, With<CameraStateTag>>,
    transforms: Query<&Transform, Without<CameraStateTag>>, 
) {
    let Ok(mut camera_state) = camera_state_query.single_mut() else {
        return;
    };

    if camera_state.control_state == CameraControlState::MOVE {
        let speed = 20.0 * time.delta_secs();
        let mut movement = Vec3::ZERO;

        if keyboard.pressed(KeyCode::KeyW) { movement.z -= speed; }
        if keyboard.pressed(KeyCode::KeyS) { movement.z += speed; }
        if keyboard.pressed(KeyCode::KeyA) { movement.x -= speed; }
        if keyboard.pressed(KeyCode::KeyD) { movement.x += speed; }

        // Break lock if the player manually moves
        if movement != Vec3::ZERO {
            camera_state.focus_target = None;
            camera_state.focus_point += movement;
        }

        // Continuously update focus point if locked onto a target
        if let Some(target_entity) = camera_state.focus_target {
            if let Ok(target_transform) = transforms.get(target_entity) {
                camera_state.focus_point = target_transform.translation;
            } else {
                camera_state.focus_target = None;
            }
        }
        
        // Handle scroll wheel for zooming
        let mut scroll_amount = 0.0;
        for ev in evr_scroll.read() {
            match ev.unit {
                MouseScrollUnit::Line => scroll_amount -= ev.y,
                MouseScrollUnit::Pixel => scroll_amount -= ev.y * 0.05,
            }
        }
        
        if scroll_amount != 0.0 {
            camera_state.zoom += scroll_amount * 5.0; 
            // Clamp the zoom
            camera_state.zoom = camera_state.zoom.clamp(5.0, 150.0);
        }
    }
}