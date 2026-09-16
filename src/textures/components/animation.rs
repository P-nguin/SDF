use bevy::prelude::*;

#[derive(Component)]
pub struct AnimationState {
    pub timer: Timer,
    pub current_frame: usize,
    pub frame_durations: Vec<u32>,
}
