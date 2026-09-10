use bevy::prelude::*;

#[derive(Component)]
pub struct MoveTo {
    pub target: Vec2,
    pub rad: f32,
}