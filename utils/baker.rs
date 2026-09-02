use std::fs::File;
use std::io::Write;
use mesh_to_sdf::*; 

const SDF_GRID_SIZE: i32 = 64;
const SDF_WORLD_SIZE: f32 = 16.0;

fn main() {
    let input_path = "assets/models/type-730_ciws.glb";
    let output_path = "assets/models/type-730_ciws.sdf";
    
    println!("1. Loading GLB from {}...", input_path);
    
    // 2. Explicitly call the root ::gltf crate to avoid any Bevy shadowing
    let (document, buffers, _) = gltf::import(input_path).expect("Failed to load GLB");

    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            
            let base_index = vertices.len() as u32;
            
            if let Some(iter) = reader.read_positions() {
                for vertex in iter {
                    vertices.push(vertex);
                }
            }
            
            if let Some(read_indices) = reader.read_indices() {
                for index in read_indices.into_u32() {
                    indices.push(base_index + index);
                }
            }
        }
    }
    println!("   -> Extracted {} vertices and {} indices.", vertices.len(), indices.len());

    println!("2. Generating {}x{}x{} voxel grid...", SDF_GRID_SIZE, SDF_GRID_SIZE, SDF_GRID_SIZE);
    
    let size = SDF_GRID_SIZE;
    let voxel_size = SDF_WORLD_SIZE / (size - 1) as f32;
    let grid_center = (size - 1) as f32 * 0.5;

    let volume = (size * size * size) as usize;
    let mut query_points = Vec::with_capacity(volume);

    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                let px = (x as f32 - grid_center) * voxel_size;
                let py = (y as f32 - grid_center) * voxel_size;
                let pz = (z as f32 - grid_center) * voxel_size;
                query_points.push([px, py, pz]);
            }
        }
    }

    println!("3. Baking SDF Math (this may take a moment)...");
    
    // 3. Pass the required AccelerationMethod as the 4th argument
    let distances = generate_sdf(
        &vertices,
        Topology::TriangleList(Some(&indices)),
        &query_points,
        AccelerationMethod::RtreeBvh,
    );

    println!("4. Writing to disk...");
    
    let mut file = File::create(output_path).expect("Failed to create output file");
    let bytes: &[u8] = bytemuck::cast_slice(&distances);
    file.write_all(bytes).expect("Failed to write bytes");

    println!("Success! SDF saved to {}", output_path);
}