use crate::voxel::components::*;
use crate::voxel::compute::ExtractedSDF;
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

            // 1. THE DUMMY MESH 
            // Give Bevy a tiny invisible mesh at spawn so it successfully registers 
            // the entity into the PBR render pipeline.
            let mut dummy_mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            );
            dummy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]]);
            dummy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]]);
            dummy_mesh.insert_indices(Indices::U32(vec![0, 0, 0]));
            
            let dummy_handle = meshes.add(dummy_mesh);

            let visibility = match layer.kind {
                ShipMeshKind::Exterior => Visibility::Visible,
                ShipMeshKind::Interior => Visibility::Hidden,
            };

            let visual_child = commands.spawn((
                Name::new(format!("{:?} Ship Mesh (Visual)", layer.kind)),
                GeneratedShipMesh {
                    ship: entity,
                    kind: layer.kind,
                },
                Mesh3d(dummy_handle), // The crucial lock for the render pipeline
                MeshMaterial3d(layer.material.clone()),
                Transform::default(),
                visibility,
                RenderLayers::layer(0), 
            )).id();

            // 2. TRIGGER THE COMPUTE SHADER
            if layer.kind == ShipMeshKind::Exterior {
                commands.entity(visual_child).insert(ExtractedSDF {
                    distances: chunk.clone(),
                });
            }

            // 3. ASCII TILEMAP GENERATOR (Remains untouched)
            let mut ascii_positions: Vec<[f32; 3]> = Vec::new();
            let mut ascii_normals: Vec<[f32; 3]> = Vec::new();
            let mut ascii_uvs: Vec<[f32; 2]> = Vec::new();
            let mut ascii_indices: Vec<u32> = Vec::new();

            let y = SDF_GRID_SIZE / 2; 

            for z in 0..SDF_GRID_SIZE {
                for x in 0..SDF_GRID_SIZE {
                    let index = (z * SDF_GRID_SIZE * SDF_GRID_SIZE + y * SDF_GRID_SIZE + x) as usize;
                    
                    if chunk[index] <= 0.0 {
                        let v_idx = ascii_positions.len() as u32;
                        let px = (x as f32 - mesh_center) * voxel_size;
                        let pz = (z as f32 - mesh_center) * voxel_size;
                        let half_size = voxel_size * 0.5;

                        ascii_positions.push([px - half_size, 0.0, pz - half_size]);
                        ascii_positions.push([px + half_size, 0.0, pz - half_size]);
                        ascii_positions.push([px + half_size, 0.0, pz + half_size]);
                        ascii_positions.push([px - half_size, 0.0, pz + half_size]);
                        ascii_normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);

                        let char_index: f32 = 35.0;
                        let font_cols = 12.0;
                        let uv_min_x = (char_index % font_cols) / font_cols;
                        let uv_min_y = (char_index / font_cols).floor() / font_cols;
                        let uv_max_x = uv_min_x + (1.0 / font_cols);
                        let uv_max_y = uv_min_y + (1.0 / font_cols);

                        ascii_uvs.push([uv_min_x, uv_min_y]);
                        ascii_uvs.push([uv_max_x, uv_min_y]);
                        ascii_uvs.push([uv_max_x, uv_max_y]);
                        ascii_uvs.push([uv_min_x, uv_max_y]);

                        ascii_indices.extend_from_slice(&[
                            v_idx, v_idx + 2, v_idx + 1,
                            v_idx, v_idx + 3, v_idx + 2
                        ]);
                    }
                }
            }

            let mut ascii_mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            );
            ascii_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, ascii_positions);
            ascii_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, ascii_normals);
            ascii_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, ascii_uvs);
            ascii_mesh.insert_indices(Indices::U32(ascii_indices));

            let ascii_mesh_handle = meshes.add(ascii_mesh);

            let data_child = commands.spawn((
                Name::new(format!("{:?} Ship Mesh (Data)", layer.kind)),
                GeneratedShipMesh {
                    ship: entity,
                    kind: layer.kind,
                },
                Mesh3d(ascii_mesh_handle), 
                MeshMaterial3d(layer.data_material.clone()),
                Transform::default(),
                visibility,
                RenderLayers::layer(1), 
            )).id();

            // 4. Attach both to the parent ship
            commands.entity(entity).add_child(visual_child);
            commands.entity(entity).add_child(data_child);
        }

        commands.entity(entity).remove::<NeedsRemesh>();
    }
}