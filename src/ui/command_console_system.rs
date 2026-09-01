use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::camera::{CameraState, CameraStateTag, CameraControlState};

pub fn command_console_system(
    mut contexts: EguiContexts, 
    mut command_buffer: Local<String>,
    camera_state_query: Query<&CameraState, With<CameraStateTag>>,
) {
    // 1. Extract the context using the safe wrapper
    let Ok(ctx) = contexts.ctx_mut() else { return; };

    // 2. Only proceed if the camera is in COMMAND mode
    let Ok(state) = camera_state_query.single() else { return; };
    
    // TEST WINDOW: ALWAYS VISIBLE
    egui::Window::new("Test UI Render")
        .show(ctx, |ui| {
            ui.label("If you can see this, EGUI is working!");
        });

    if state.control_state != CameraControlState::COMMAND {
        return;
    }

    // 3. Draw the interactive window
    egui::Window::new("Terminal")
        .default_size([400.0, 200.0])
        .default_pos([10.0, 10.0])
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            // ui.label("System ready. Awaiting orders...");
            // ui.separator();

            let response = ui.add(
                egui::TextEdit::singleline(&mut *command_buffer)
                    // .hint_text("e.g., move_to 10 0 -20")
                    .desired_width(f32::INFINITY)
            );

            // Handle the Enter Key
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let command = command_buffer.trim().to_string();
                if !command.is_empty() {
                    info!("Command Issued: {}", command); 
                }
                
                command_buffer.clear();
                response.request_focus();
            }
        });
}