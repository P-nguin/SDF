use bevy::prelude::*;
use std::f32::consts::FRAC_PI_4;
use crate::camera::components::{CameraState, CameraStateTag};

pub fn sync_camera_transform(
    mut camera_query: Query<(&mut Transform, &CameraState), With<CameraStateTag>>
) {
    let Ok((mut transform, state)) = camera_query.single_mut() else {
        return;
    };

    let fixed_rotation = Quat::from_euler(EulerRot::YXZ, 0.0, -FRAC_PI_4, 0.0);
    let backward_direction = fixed_rotation * Vec3::Z;

    transform.translation = state.focus_point + (backward_direction * state.zoom);
    transform.rotation = fixed_rotation;
}