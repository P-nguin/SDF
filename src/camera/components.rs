use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CameraRenderState {
    ASCII, 
    VOXEL
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CameraControlState {
    MOVE,
    COMMAND
}

#[derive(Component)]
pub struct CameraStateTag;

#[derive(Component)]
pub struct CameraState {
    pub render_state: CameraRenderState,
    pub control_state: CameraControlState,

    pub zoom: f32,
    pub zoom_threshold: f32,

    pub focus_point: Vec3,
    pub focus_target: Option<Entity>
}

#[derive(Component)]
pub struct Selected;