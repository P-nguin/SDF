use avian3d::prelude::RigidBody;
use bevy::prelude::*;

use crate::voxel::{GenerateMesh, *};

// marker
#[derive(Component)]
pub struct ShipTag;

// actual ship struct
#[derive(Bundle)]
pub struct ShipBundle {
    pub ship_tag: ShipTag,
    pub transform: Transform,
    pub rigidbody: RigidBody,

    pub generate_mesh: GenerateMesh,
    pub chunks: SDFChunk,
}
