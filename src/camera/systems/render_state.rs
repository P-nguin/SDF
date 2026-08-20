use bevy::prelude::*;
use crate::camera::components::{CameraStateTag, CameraState, CameraRenderState};

pub fn determine_render_state(mut camera_state_query:Query<&mut CameraState, With<CameraStateTag>>) {
    let Ok(mut camera_state) = camera_state_query.single_mut() else {
        return;
    };

    if camera_state.zoom >= camera_state.zoom_threshold {
        camera_state.render_state = CameraRenderState::ASCII;
    }
    else {
        camera_state.render_state = CameraRenderState::VOXEL;
    }
}