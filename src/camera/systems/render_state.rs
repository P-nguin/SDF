use bevy::prelude::*;
use bevy::camera::ClearColorConfig;
use crate::camera::components::{CameraStateTag, CameraState, CameraRenderState};
use crate::ascii::AsciiSettings;

pub fn determine_render_state(
    mut camera_state_query: Query<(
        &mut CameraState, 
        &mut AsciiSettings,
        &mut Msaa,
        &mut Camera
    ), With<CameraStateTag>>
) {
    let Ok((mut camera_state, mut ascii_settings, mut msaa, mut camera)) = camera_state_query.single_mut() else {
        return;
    };

    let target_state = if camera_state.zoom >= camera_state.zoom_threshold {
        CameraRenderState::ASCII
    } else {
        CameraRenderState::VOXEL
    };

    if camera_state.render_state != target_state {
        camera_state.render_state = target_state;

        if target_state == CameraRenderState::ASCII {
            ascii_settings.enabled = true;
            *msaa = Msaa::Off;
            camera.clear_color = ClearColorConfig::Custom(Color::BLACK);
        } else {
            ascii_settings.enabled = false;
            *msaa = Msaa::Sample4;
            camera.clear_color = ClearColorConfig::Default;
        }
    }
}