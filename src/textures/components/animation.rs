use bevy::prelude::*;

#[derive(Component)]
pub struct AnimationState {
    pub timer: Timer,
    pub current_index: usize,
    pub frame_indices: Vec<usize>,
    pub frame_durations: Vec<u32>,
}

impl AnimationState {
    pub fn set_part(&mut self, part: &crate::textures::components::chunk_registry::SpritePart) {
        self.frame_indices = part.frame_indices.clone();
        self.frame_durations = part.frame_durations.clone();
        self.current_index = 0;
        if !self.frame_durations.is_empty() {
            self.timer.set_duration(std::time::Duration::from_millis(self.frame_durations[0] as u64));
            self.timer.reset();
        }
    }
}
