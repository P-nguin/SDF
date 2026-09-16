// In src/camera/plugins/camera_plugin.rs
use bevy::prelude::*;
use crate::camera::systems::camera_control_system;

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_control_system);
    }
}