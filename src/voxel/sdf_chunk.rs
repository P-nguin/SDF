use bevy::prelude::*;

pub const SDF_GRID_SIZE: i32 = 64;
pub const SDF_WORLD_SIZE: f32 = 16.0;

#[derive(Component)]
pub struct SDFChunks {
    pub layers: Vec<SDFLayer>,
}

pub struct SDFLayer {
    pub kind: ShipMeshKind,
    pub distances: Vec<f32>,
    pub material: Handle<StandardMaterial>,
    // pub chunk_index: IVec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShipMeshKind {
    Exterior,
    Interior,
}

#[derive(Component)]
pub struct GeneratedShipMesh {
    pub ship: Entity,
    pub kind: ShipMeshKind,
}

#[derive(Component)]
pub struct NeedsRemesh {
    pub exterior: bool,
    pub interior: bool,
}

impl NeedsRemesh {
    pub fn all() -> Self {
        Self {
            exterior: true,
            interior: true,
        }
    }

    pub fn should_remesh(&self, kind: ShipMeshKind) -> bool {
        match kind {
            ShipMeshKind::Exterior => self.exterior,
            ShipMeshKind::Interior => self.interior,
        }
    }
}
