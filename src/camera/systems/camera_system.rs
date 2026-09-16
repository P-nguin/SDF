// In src/camera/systems/camera_system.rs
use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use crate::camera::components::CameraState;

pub fn camera_control_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut scroll_evr: MessageReader<MouseWheel>,
    mut motion_evr: MessageReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut Projection, &mut CameraState)>,
) {
    // Only run if we have a camera with our custom state
    let Ok((mut transform, mut projection, mut state)) = query.single_mut() else { return };
    
    let dt = time.delta_secs();

    // 1. Keyboard Panning Controls (WASD or Arrows)
    let mut pan_direction = Vec2::ZERO;
    
    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        pan_direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        pan_direction.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        pan_direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        pan_direction.x += 1.0;
    }

    // Apply keyboard pan translation
    if pan_direction.length_squared() > 0.0 {
        pan_direction = pan_direction.normalize();
        let pan_speed = 500.0 * state.zoom; 
        transform.translation += pan_direction.extend(0.0) * pan_speed * dt;
    }

    // 2. Middle-Mouse Panning Controls
    // We always read the motion events to prevent them from buffering up in the background
    let mut mouse_delta = Vec2::ZERO;
    for ev in motion_evr.read() {
        mouse_delta += ev.delta;
    }
    
    if mouse_input.pressed(MouseButton::Middle) {
        // Multiply by zoom so panning matches the visual scale
        transform.translation.x -= mouse_delta.x * state.zoom;
        // Bevy's Y goes up, mouse Y goes down, so we add instead of subtract
        transform.translation.y += mouse_delta.y * state.zoom; 
    }

    // 3. Zoom Controls (Mouse Wheel)
    let mut zoom_delta = 0.0;
    for ev in scroll_evr.read() {
        match ev.unit {
            MouseScrollUnit::Line => zoom_delta -= ev.y * 0.1,
            MouseScrollUnit::Pixel => zoom_delta -= ev.y * 0.001,
        }
    }

    if zoom_delta != 0.0 {
        state.zoom *= 1.0 + zoom_delta;
        state.zoom = state.zoom.clamp(0.2, 5.0); 
        
        // Match on the generic Projection enum to safely apply the scale
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale = state.zoom;
        }
    }
}