use avian3d::prelude::RigidBody;
use bevy::prelude::*;

use crate::voxel::{NeedsRemesh, SDFChunks};

// marker
#[derive(Component)]
pub struct Ship;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShipRenderMode {
    Exterior,
    Interior,
    Both,
}

#[derive(Component)]
pub struct ShipRender {
    pub mode: ShipRenderMode,
}

// actual ship struct
#[derive(Bundle)]
pub struct ShipBundle {
    pub marker: Ship,
    pub transform: Transform,
    pub rigidbody: RigidBody,

    pub render: ShipRender,
    pub needs_remesh: NeedsRemesh,
    pub chunks: SDFChunks,
}
