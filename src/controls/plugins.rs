use bevy::prelude::*;
use crate::controls::components::*;
use crate::controls::systems::*;
use crate::AppState;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionState>()
           .init_resource::<CommandMode>()
           .add_systems(Update, (
               handle_command_mode_input,
               handle_selection_input,
               render_selection_box_system,
               render_selected_units_system,
               render_targeting_ui_system,
           ).chain().run_if(in_state(AppState::InGame)));
    }
}
