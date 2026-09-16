use bevy::prelude::*;

use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct CellRegistry {
    pub cells: HashMap<String, Cell>,
}

#[derive(Component, Clone)]
pub struct Cell {
    pub cell_type: String,

    pub health: f32,
    pub max_health: f32,

    pub minus: f32,

    pub give_plus: f32,
    pub plus: f32,

    pub shield: f32,
    pub max_shield: f32,
}
impl Cell {
    pub fn from_registry(cell_type: &str, registry: &CellRegistry) -> Cell {
        registry.cells[cell_type].clone()
    }
    pub fn new(
        cell_type: String,
        health: f32,
        max_health: f32,
        minus: f32,
        give_plus: f32,
        plus: f32,
        shield: f32,
        max_shield: f32,
    ) -> Cell {
        Cell {
            cell_type,
            health,
            max_health,
            minus,
            give_plus,
            plus,
            shield,
            max_shield,
        }
    }
}

#[derive(Component)]
pub struct CellPos {
    pub x: i16, 
    pub y:i16,
}

#[derive(Component)]
pub struct PlusCell;

#[derive(Component)]
pub struct MinusCell;