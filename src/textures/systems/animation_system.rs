use bevy::prelude::*;
use crate::textures::components::AnimationState;

pub fn animate_sprites_system(
    time: Res<Time>,
    mut query: Query<(&mut AnimationState, &mut Sprite)>,
) {
    for (mut state, mut sprite) in query.iter_mut() {
        state.timer.tick(time.delta());
        
        if state.timer.just_finished() {
            if state.frame_durations.is_empty() {
                continue;
            }
            
            state.current_frame = (state.current_frame + 1) % state.frame_durations.len();
            
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = state.current_frame;
            }
            
            let next_duration_ms = state.frame_durations[state.current_frame];
            state.timer.set_duration(std::time::Duration::from_millis(next_duration_ms as u64));
            state.timer.reset(); 
        }
    }
}
