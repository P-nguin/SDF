use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Waypoint {
    pub position: Vec2,          // Where the ship should be
    pub target_speed: f32,       // How fast it should be moving when it gets there
    pub target_heading: Vec2,    // Where it should be pointing when it gets there
    pub radius: f32,             // Arrival tolerance
}

#[derive(Component, Default, Debug, Clone)]
pub struct NavigationPlan {
    pub waypoints: VecDeque<Waypoint>,
}
