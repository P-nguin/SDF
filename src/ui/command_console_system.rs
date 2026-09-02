use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::camera::{CameraState, CameraStateTag, CameraControlState, Selected};
use crate::ship::NavTarget;

pub fn command_console_system(
    mut commands: Commands,
    mut contexts: EguiContexts, 
    mut command_buffer: Local<String>,
    camera_state_query: Query<&CameraState, With<CameraStateTag>>,
    selected_ships: Query<Entity, With<Selected>>, 
) {
    let Ok(state) = camera_state_query.single() else { return; };
    if state.control_state != CameraControlState::COMMAND {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return; };

    egui::Window::new("Terminal [ESC to close]")
        .default_size([400.0, 200.0])
        .default_pos([10.0, 10.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("System ready. Awaiting orders...");
            
            let response = ui.add(
                egui::TextEdit::singleline(&mut *command_buffer)
                    .hint_text("e.g., m_10_-20")
                    .desired_width(f32::INFINITY)
            );

            // FORCE FOCUS: All typing routes here immediately
            response.request_focus();

            // FIX: Remove `response.lost_focus()`, just check for the Enter key
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let command = command_buffer.trim().to_lowercase();
                let parts: Vec<&str> = command.split('_').collect();

                match parts.as_slice() {
                    ["m", x_str, z_str] => {
                        if let (Ok(x), Ok(z)) = (x_str.parse::<f32>(), z_str.parse::<f32>()) {
                            let target = Vec3::new(x, 0.0, z);
                            let mut count = 0;
                            
                            for entity in selected_ships.iter() {
                                commands.entity(entity).insert(NavTarget(target));
                                count += 1;
                            }
                            info!("Dispatched {} ship(s) to X:{} Z:{}", count, x, z);
                        } else {
                            warn!("Invalid coordinates: {}", command);
                        }
                    },
                    // Prevent warning on empty submission
                    [""] => {}, 
                    _ => warn!("Unknown command: {}", command),
                }
                
                // Clear the buffer after sending the command
                command_buffer.clear();
            }
        });
}