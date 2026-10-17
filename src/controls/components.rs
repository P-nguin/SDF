use bevy::prelude::*;

#[derive(Component)]
pub struct Selected;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Targetable;

#[derive(Resource, Default)]
pub struct SelectionState {
    pub is_dragging: bool,
    pub start_pos: Vec2,
    pub current_pos: Vec2,
}

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum CommandMode {
    #[default]
    Normal,
    Targeting,
}
