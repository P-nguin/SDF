use bevy::prelude::*;
use bevy::camera::visibility::RenderLayers;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};
use crate::camera::components::{CameraState, CameraStateTag, CameraRenderState};

pub fn sync_camera_transform(
    mut camera_query: Query<(&mut Transform, &mut RenderLayers, &CameraState), With<CameraStateTag>>
) {
    let Ok((mut transform, mut render_layers, state)) = camera_query.single_mut() else {
        return;
    };

    let (pitch, target_layer) = match state.render_state {
        CameraRenderState::VOXEL => (-FRAC_PI_4, 0), // 45 degrees, Layer 0
        CameraRenderState::ASCII => (-FRAC_PI_2, 1), // 90 degrees straight down, Layer 1
    };

    // Apply the layer switch
    *render_layers = RenderLayers::layer(target_layer);

    // Apply the transform math using the dynamic pitch
    let fixed_rotation = Quat::from_euler(EulerRot::YXZ, 0.0, pitch, 0.0);
    let backward_direction = fixed_rotation * Vec3::Z;
    
    transform.translation = state.focus_point + (backward_direction * state.zoom);
    transform.rotation = fixed_rotation;
}