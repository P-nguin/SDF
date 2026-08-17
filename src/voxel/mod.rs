mod generate_mesh;
mod sdf_chunk;

pub use generate_mesh::generate_chunk_mesh_system;
pub use sdf_chunk::{
    GeneratedShipMesh, NeedsRemesh, SDF_GRID_SIZE, SDF_WORLD_SIZE, SDFChunks, SDFLayer,
    ShipMeshKind,
};
