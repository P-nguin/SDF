use bevy::prelude::*;
use bevy::camera::ClearColorConfig;
use crate::camera::components::{CameraStateTag, CameraState, CameraRenderState};
use bevy::camera::visibility::RenderLayers;
pub fn determine_render_state(
    mut camera_state_query: Query<(
        &mut CameraState, 
        &mut RenderLayers,
        &mut Msaa,
        &mut Camera
    ), With<CameraStateTag>>
) {
    let Ok((mut camera_state, mut render_layers, mut msaa, mut camera)) = camera_state_query.single_mut() else {
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
            *render_layers = RenderLayers::layer(1);
            *msaa = Msaa::Off;
            camera.clear_color = ClearColorConfig::Custom(Color::BLACK);
        } else {
            *render_layers = RenderLayers::layer(0);
            *msaa = Msaa::Off;
            camera.clear_color = ClearColorConfig::Default;
        }
    }
}