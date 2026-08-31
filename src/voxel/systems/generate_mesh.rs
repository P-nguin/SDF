use crate::voxel::components::*;
use crate::voxel::compute::ExtractedSDF;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    camera::visibility::{RenderLayers, NoFrustumCulling},
    camera::primitives::Aabb,
};

pub fn generate_chunk_mesh_system(
    mut commands: Commands,
    query: Query<(Entity, &SDFChunk), With<GenerateMesh>>, // querying all entities with a GenerateMesh tag
    mut meshes: ResMut<Assets<Mesh>>, // this holds all Meshes and we use this to store the padded and ascii meshs
) {
    for (entity, chunk) in query.iter() {
        let mesh_center = (SDF_GRID_SIZE - 1) as f32 * 0.5;
        let voxel_size = SDF_WORLD_SIZE / (SDF_GRID_SIZE - 1) as f32;

        // determine the max size of the mesh to allocate a buffer for it
        // this mesh is empty
        // but once copied onto the GPU we can overwrite the data to create the actual mesh
        // this lets us use bevy's own lighting and materials instead of having to write own renderer
        let volume = (SDF_GRID_SIZE * SDF_GRID_SIZE * SDF_GRID_SIZE) as usize;
        let max_indices = volume * 18; 
        let mut max_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        max_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0; 3]; volume]);
        max_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0; 3]; volume]);
        max_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0; 2]; volume]);
        max_mesh.insert_indices(Indices::U32(vec![0; max_indices]));

        let max_mesh_handle = meshes.add(max_mesh); // gives us a handle which references back to this mesh
        // let mesh_id = mesh_handle.id();

        // ---- useless ----
        /*
            useless for now as it doesnt do anything
            we currently depend on GPU culling which makes it so that if the ceneter of a entity is not in screen then we dont render the mesh
            this only saves draw time
            but we still incur costs on drawing nonvisible quads and mesh creation costs
            no idea how aabb works
         */
        let half_extent = SDF_WORLD_SIZE * 0.01;
        let aabb = Aabb::from_min_max(
            Vec3::splat(-half_extent),
            Vec3::splat(half_extent),
        );

        // we are creating the visual render package for bevy
        let visual_child = commands.spawn((
            Name::new("Ship Mesh (Visual)"),
            GeneratedShipMesh { // consider cleaning this struct up later 
                ship: entity,
            },
            Mesh3d(max_mesh_handle.clone()), // notice that we are tying this to the max_mesh so that when we overwrite the mesh in the GPU it overwrites the correct mesh
            MeshMaterial3d(chunk.material.clone()),
            Transform::default(),
            Visibility::Visible,
            RenderLayers::layer(0),
            aabb, // again useless stuff
            NoFrustumCulling, // again useless stuff
        )).id();

        // prepare the data package that we are sending to the render world which will make its way to the GPU
        // 
        commands.entity(visual_child).insert(ExtractedSDF {
            mesh_id: max_mesh_handle.id().clone(), // again we pass in the mesh_id we do this so that from the GPU we know which asset to modify
            distances: chunk.distances.clone(),
        });

        // ASCII TILEMAP GENERATOR
        /*
         look into this code below later
         it seems to work so hurray!
        */
        let mut ascii_positions: Vec<[f32; 3]> = Vec::new();
        let mut ascii_normals: Vec<[f32; 3]> = Vec::new();
        let mut ascii_uvs: Vec<[f32; 2]> = Vec::new();
        let mut ascii_indices: Vec<u32> = Vec::new();

        let y = SDF_GRID_SIZE / 2; 

        for z in 0..SDF_GRID_SIZE {
            for x in 0..SDF_GRID_SIZE {
                let index = (z * SDF_GRID_SIZE * SDF_GRID_SIZE + y * SDF_GRID_SIZE + x) as usize;
                
                if chunk.distances[index] <= 0.0 {
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
            Name::new("Ship Mesh (Data)"),
            GeneratedShipMesh {
                ship: entity,
            },
            Mesh3d(ascii_mesh_handle), 
            MeshMaterial3d(chunk.data_material.clone()),
            Transform::default(),
            Visibility::Visible,
            RenderLayers::layer(1), 
        )).id();

        // 4. Attach both to the parent ship
        commands.entity(entity).add_child(visual_child);
        commands.entity(entity).add_child(data_child);

        commands.entity(entity).remove::<GenerateMesh>();
    }
}