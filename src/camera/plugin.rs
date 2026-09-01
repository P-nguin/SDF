use bevy::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::ClearColorConfig;

use bevy::core_pipeline::prepass::DepthPrepass;

use crate::camera::components::{CameraState, CameraStateTag, CameraRenderState, CameraControlState};
use crate::camera::systems::handle_input;
use crate::camera::systems::determine_render_state;
use crate::camera::systems::sync_camera_transform;
use crate::camera::systems::select_ship_with_mouse;
use crate::camera::systems::focus_on_selected_ship;


pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, handle_input)
            .add_systems(Update, sync_camera_transform)
            .add_systems(Update, determine_render_state);
    }
}

fn spawn_camera(mut commands: Commands) { 

    commands.spawn((
        Camera3d::default(),
        Camera::default(), 
        Transform::default(),
        RenderLayers::layer(0),
        DepthPrepass,
        Msaa::Off, 
        CameraStateTag,
        CameraState {
            render_state: CameraRenderState::VOXEL,
            control_state: CameraControlState::COMMAND,
            zoom: 30.0,
            zoom_threshold: 60.0,
            focus_point: Vec3::ZERO,
            focus_target: None,
        },
    ));
}

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (select_ship_with_mouse, focus_on_selected_ship));
    }
}