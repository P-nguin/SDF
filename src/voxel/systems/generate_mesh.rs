use crate::voxel::components::*;
use crate::qef::QEFData;
use avian3d::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    camera::visibility::RenderLayers,
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

                        // X-axis edge
                        if (d0 <= 0.0) != (get_distance(x + 1, y, z, chunk) <= 0.0) {
                            if let (Some(v1), Some(v2), Some(v3), Some(v4)) = (
                                get_vidx(x, y, z),
                                get_vidx(x, y - 1, z),
                                get_vidx(x, y - 1, z - 1),
                                get_vidx(x, y, z - 1),
                            ) {
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

            let collider_vertices: Vec<Vec3> = positions
                .iter()
                .map(|p| Vec3::new(p[0], p[1], p[2]))
                .collect();
            let collider_indices: Vec<[u32; 3]> = indices
                .chunks_exact(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect();
            let collider = Collider::try_trimesh(collider_vertices, collider_indices);

            let visibility = match layer.kind {
                ShipMeshKind::Exterior => Visibility::Visible,
                ShipMeshKind::Interior => Visibility::Hidden,
            };

            // 1. Get the single mesh pointer
            let mesh_handle = meshes.add(mesh);

            // 2. Spawn Child A: The Visual Mesh (Layer 0)
            let visual_child = {
                let mut child_commands = commands.spawn((
                    Name::new(format!("{:?} Ship Mesh (Visual)", layer.kind)),
                    GeneratedShipMesh {
                        ship: entity,
                        kind: layer.kind,
                    },
                    Mesh3d(mesh_handle.clone()), // Use clone() to reuse the pointer
                    MeshMaterial3d(layer.material.clone()),
                    Transform::default(),
                    visibility,
                    RenderLayers::layer(0), // ONLY visible to Layer 0 cameras
                ));

                if layer.kind == ShipMeshKind::Exterior {
                    if let Ok(col) = &collider {
                        child_commands.insert(col.clone());
                    } else if let Err(error) = &collider {
                        warn!("Failed to build collider for exterior ship mesh: {error:?}");
                    }
                }
                child_commands.id()
            };

            // 3. Spawn Child B: The Data Mesh (Layer 1)
            let data_child = commands.spawn((
                Name::new(format!("{:?} Ship Mesh (Data)", layer.kind)),
                GeneratedShipMesh {
                    ship: entity,
                    kind: layer.kind,
                },
                Mesh3d(mesh_handle), // Point to the exact same geometry!
                MeshMaterial3d(layer.data_material.clone()),
                Transform::default(),
                visibility,
                RenderLayers::layer(1), // ONLY visible to Layer 1 cameras
            )).id();

            // 4. Attach both to the parent ship
            commands.entity(entity).add_child(visual_child);
            commands.entity(entity).add_child(data_child);
        }
        commands.entity(entity).remove::<NeedsRemesh>();
    }
}

fn find_intersections(x: i32, y: i32, z: i32, chunk: &[f32], qef: &mut QEFData) {
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

    let edges = [
        [0, 1], [1, 2], [2, 3], [3, 0],
        [4, 5], [5, 6], [6, 7], [7, 4],
        [0, 4], [1, 5], [2, 6], [3, 7],
    ];

    for edges in edges.iter() {
        let c1 = corners[edges[0]];
        let c2 = corners[edges[1]];

        let d1 = get_distance(c1[0], c1[1], c1[2], chunk);
        let d2 = get_distance(c2[0], c2[1], c2[2], chunk);

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

fn get_distance(x: i32, y: i32, z: i32, chunk: &[f32]) -> f32 {
    if x < 0 || y < 0 || z < 0 || x >= SDF_GRID_SIZE || y >= SDF_GRID_SIZE || z >= SDF_GRID_SIZE {
        return 1.0;
    }
    chunk[(z * SDF_GRID_SIZE * SDF_GRID_SIZE + y * SDF_GRID_SIZE + x) as usize]
}

fn get_normal(x: f32, y: f32, z: f32, chunk: &[f32]) -> [f32; 3] {
    let ix = x.round() as i32;
    let iy = y.round() as i32;
    let iz = z.round() as i32;

    let nx = get_distance(ix + 1, iy, iz, chunk) - get_distance(ix - 1, iy, iz, chunk);
    let ny = get_distance(ix, iy + 1, iz, chunk) - get_distance(ix, iy - 1, iz, chunk);
    let nz = get_distance(ix, iy, iz + 1, chunk) - get_distance(ix, iy, iz - 1, chunk);

    let n = Vec3::new(nx, ny, nz).normalize_or_zero();
    [n.x, n.y, n.z]
}