use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use crate::ui::*;

pub struct CommandUIPlugin;

impl Plugin for CommandUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, command_console_system);
    }
}