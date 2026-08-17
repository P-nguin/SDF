use super::sdf_chunk::{
    GeneratedShipMesh, NeedsRemesh, SDF_GRID_SIZE, SDF_WORLD_SIZE, SDFChunks, ShipMeshKind,
};
use crate::qef::QEFData;
use avian3d::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

pub fn generate_chunk_mesh_system(
    mut commands: Commands,
    query: Query<(Entity, &SDFChunks, &NeedsRemesh)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, chunks, needs_remesh) in query.iter() {
        for layer in chunks.layers.iter() {
            if !needs_remesh.should_remesh(layer.kind) {
                continue;
            }

            let mesh_center = (SDF_GRID_SIZE - 1) as f32 * 0.5;
            let voxel_size = SDF_WORLD_SIZE / (SDF_GRID_SIZE - 1) as f32;
            let chunk = &layer.distances;
            let mut positions: Vec<[f32; 3]> = Vec::new();
            let mut normals: Vec<[f32; 3]> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();
            let mut vertex_indices =
                vec![-1; (SDF_GRID_SIZE * SDF_GRID_SIZE * SDF_GRID_SIZE) as usize];

            // solve QEF for each cell
            for z in 0..(SDF_GRID_SIZE - 1) {
                for y in 0..(SDF_GRID_SIZE - 1) {
                    for x in 0..(SDF_GRID_SIZE - 1) {
                        let mut qef = QEFData::new();

                        find_intersections(x, y, z, chunk, &mut qef);

                        // If we found surface intersections in this cell, solve the QEF
                        if qef.intersection_count > 0.0 {
                            let solved_vertex = qef.solve();
                            let optimal_vertex = [
                                solved_vertex[0].clamp(x as f32, x as f32 + 1.0),
                                solved_vertex[1].clamp(y as f32, y as f32 + 1.0),
                                solved_vertex[2].clamp(z as f32, z as f32 + 1.0),
                            ];

                            let v_idx = positions.len() as i32;
                            positions.push([
                                (optimal_vertex[0] - mesh_center) * voxel_size,
                                (optimal_vertex[1] - mesh_center) * voxel_size,
                                (optimal_vertex[2] - mesh_center) * voxel_size,
                            ]);

                            // For normals on the final mesh, we can re-sample the normal at the optimal vertex
                            normals.push(get_normal(
                                optimal_vertex[0],
                                optimal_vertex[1],
                                optimal_vertex[2],
                                chunk,
                            ));

                            vertex_indices[(z * SDF_GRID_SIZE * SDF_GRID_SIZE
                                + y * SDF_GRID_SIZE
                                + x) as usize] = v_idx;
                        }
                    }
                }
            }

            // generate quads for each point
            let get_vidx = |x: i32, y: i32, z: i32| -> Option<u32> {
                let index = vertex_indices
                    [(z * SDF_GRID_SIZE * SDF_GRID_SIZE + y * SDF_GRID_SIZE + x) as usize];
                (index >= 0).then_some(index as u32)
            };

            for z in 1..(SDF_GRID_SIZE - 1) {
                for y in 1..(SDF_GRID_SIZE - 1) {
                    for x in 1..(SDF_GRID_SIZE - 1) {
                        let d0 = get_distance(x, y, z, chunk);

                        // X-axis edge (shared by cells: current, Y-1, Z-1, Y-1 Z-1)
                        if (d0 <= 0.0) != (get_distance(x + 1, y, z, chunk) <= 0.0) {
                            if let (Some(v1), Some(v2), Some(v3), Some(v4)) = (
                                get_vidx(x, y, z),
                                get_vidx(x, y - 1, z),
                                get_vidx(x, y - 1, z - 1),
                                get_vidx(x, y, z - 1),
                            ) {
                                // Wind the quad based on which direction the normal faces
                                if d0 <= 0.0 {
                                    indices.extend_from_slice(&[v1, v2, v3, v1, v3, v4]);
                                } else {
                                    indices.extend_from_slice(&[v1, v4, v3, v1, v3, v2]);
                                }
                            }
                        }

                        // Y-axis edge
                        if (d0 <= 0.0) != (get_distance(x, y + 1, z, chunk) <= 0.0) {
                            if let (Some(v1), Some(v2), Some(v3), Some(v4)) = (
                                get_vidx(x, y, z),
                                get_vidx(x, y, z - 1),
                                get_vidx(x - 1, y, z - 1),
                                get_vidx(x - 1, y, z),
                            ) {
                                if d0 <= 0.0 {
                                    indices.extend_from_slice(&[v1, v2, v3, v1, v3, v4]);
                                } else {
                                    indices.extend_from_slice(&[v1, v4, v3, v1, v3, v2]);
                                }
                            }
                        }

                        // Z-axis edge
                        if (d0 <= 0.0) != (get_distance(x, y, z + 1, chunk) <= 0.0) {
                            if let (Some(v1), Some(v2), Some(v3), Some(v4)) = (
                                get_vidx(x, y, z),
                                get_vidx(x - 1, y, z),
                                get_vidx(x - 1, y - 1, z),
                                get_vidx(x, y - 1, z),
                            ) {
                                if d0 <= 0.0 {
                                    indices.extend_from_slice(&[v1, v2, v3, v1, v3, v4]);
                                } else {
                                    indices.extend_from_slice(&[v1, v4, v3, v1, v3, v2]);
                                }
                            }
                        }
                    }
                }
            }

            // build mesh
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.insert_indices(Indices::U32(indices.clone()));

            // convert mesh positions into the vertex format Avian expects for a trimesh collider
            let collider_vertices: Vec<Vec3> = positions
                .iter()
                .map(|p| Vec3::new(p[0], p[1], p[2]))
                .collect();

            // group the flat indices array into groups of three
            let collider_indices: Vec<[u32; 3]> = indices
                .chunks_exact(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect();

            let collider = Collider::try_trimesh(collider_vertices, collider_indices);

            let visibility = match layer.kind {
                ShipMeshKind::Exterior => Visibility::Visible,
                ShipMeshKind::Interior => Visibility::Hidden,
            };

            let child_entity = {
                let mut child_commands = commands.spawn((
                    Name::new(format!("{:?} Ship Mesh", layer.kind)),
                    GeneratedShipMesh {
                        ship: entity,
                        kind: layer.kind,
                    },
                    Mesh3d(meshes.add(mesh)),
                    Transform::default(),
                    visibility,
                ));

                child_commands.insert(MeshMaterial3d(layer.material.clone()));

                if layer.kind == ShipMeshKind::Exterior {
                    match collider {
                        Ok(collider) => {
                            child_commands.insert(collider);
                        }
                        Err(error) => {
                            warn!("Failed to build collider for exterior ship mesh: {error:?}");
                        }
                    }
                }

                child_commands.id()
            };

            commands.entity(entity).add_child(child_entity);
        }

        commands.entity(entity).remove::<NeedsRemesh>();
    }
}

fn find_intersections(x: i32, y: i32, z: i32, chunk: &[f32], qef: &mut QEFData) {
    // Define the 8 corners of this cell
    let corners = [
        [x, y, z],
        [x + 1, y, z],
        [x + 1, y + 1, z],
        [x, y + 1, z],
        [x, y, z + 1],
        [x + 1, y, z + 1],
        [x + 1, y + 1, z + 1],
        [x, y + 1, z + 1],
    ];

    // Define the 12 edges connecting those corners (by index)
    let edges = [
        [0, 1],
        [1, 2],
        [2, 3],
        [3, 0], // Bottom face
        [4, 5],
        [5, 6],
        [6, 7],
        [7, 4], // Top face
        [0, 4],
        [1, 5],
        [2, 6],
        [3, 7], // Vertical pillars
    ];

    for edges in edges.iter() {
        let c1 = corners[edges[0]];
        let c2 = corners[edges[1]];
        // for each edge we want to see if the object partitions this space (that is find the vertex that lies on this edge where we go from object -> space)
        let d1 = get_distance(c1[0], c1[1], c1[2], chunk);
        let d2 = get_distance(c2[0], c2[1], c2[2], chunk);

        // if d1 and d2 are different signs then they are on different sides of the mesh
        if (d1 <= 0.0) != (d2 <= 0.0) {
            let t = d1 / (d1 - d2);
            let px = c1[0] as f32 + t * (c2[0] - c1[0]) as f32;
            let py = c1[1] as f32 + t * (c2[1] - c1[1]) as f32;
            let pz = c1[2] as f32 + t * (c2[2] - c1[2]) as f32;
            let normal = get_normal(px, py, pz, chunk);

            qef.add_intersection([px, py, pz], normal);
        }
    }
}

// gets the sdf distance at (x, y, z)
fn get_distance(x: i32, y: i32, z: i32, chunk: &[f32]) -> f32 {
    if x < 0 || y < 0 || z < 0 || x >= SDF_GRID_SIZE || y >= SDF_GRID_SIZE || z >= SDF_GRID_SIZE {
        return 1.0;
    }
    chunk[(z * SDF_GRID_SIZE * SDF_GRID_SIZE + y * SDF_GRID_SIZE + x) as usize]
}

// estimate the normal using Central Differences
fn get_normal(x: f32, y: f32, z: f32, chunk: &[f32]) -> [f32; 3] {
    // In a real implementation, you would interpolate the distance field at exact float coordinates.
    // For simplicity, we sample the nearest integers.
    let ix = x.round() as i32;
    let iy = y.round() as i32;
    let iz = z.round() as i32;

    let nx = get_distance(ix + 1, iy, iz, chunk) - get_distance(ix - 1, iy, iz, chunk);
    let ny = get_distance(ix, iy + 1, iz, chunk) - get_distance(ix, iy - 1, iz, chunk);
    let nz = get_distance(ix, iy, iz + 1, chunk) - get_distance(ix, iy, iz - 1, chunk);

    let n = Vec3::new(nx, ny, nz).normalize_or_zero();
    [n.x, n.y, n.z]
}
