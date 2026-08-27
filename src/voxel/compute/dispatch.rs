use bevy::{
    prelude::*,
    render::{
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        extract_component::ExtractComponent,
    },
};
use std::sync::{mpsc, Mutex};
use crate::voxel::compute::buffers::DualContouringPipeline;
use crate::voxel::SDF_GRID_SIZE;

pub struct MeshReadbackData {
    pub entity: Entity,
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

#[derive(Resource)]
pub struct ComputeReceiver {
    pub rx: Mutex<mpsc::Receiver<MeshReadbackData>>,
}

#[derive(Resource, Clone)]
pub struct ComputeSender {
    pub tx: mpsc::Sender<MeshReadbackData>,
}

#[derive(Component, Clone, ExtractComponent)]
pub struct ExtractedSDF {
    pub distances: Vec<f32>,
}

#[derive(Component)]
pub struct ComputeDispatched;

pub fn dispatch_dual_contouring(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    dc_pipeline: Res<DualContouringPipeline>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    sender: Res<ComputeSender>, 
    query: Query<(Entity, &ExtractedSDF), Without<ComputeDispatched>>, 
) {
    let qef_pipeline = pipeline_cache.get_compute_pipeline(dc_pipeline.qef_pipeline_id);
    let quads_pipeline = pipeline_cache.get_compute_pipeline(dc_pipeline.quads_pipeline_id);
    
    let (Some(qef_pipeline), Some(quads_pipeline)) = (qef_pipeline, quads_pipeline) else { return; };

    for (entity, extracted_sdf) in query.iter() {
        commands.entity(entity).insert(ComputeDispatched);

        let volume = (SDF_GRID_SIZE * SDF_GRID_SIZE * SDF_GRID_SIZE) as u64;

        let sdf_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("sdf_input"),
            contents: bytemuck::cast_slice(&extracted_sdf.distances),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        let vertices_size = volume * std::mem::size_of::<[f32; 3]>() as u64;
        let vertex_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("dc_vertices"),
            size: vertices_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC, 
            mapped_at_creation: false,
        });

        let indices_size = (volume * 6) * std::mem::size_of::<u32>() as u64;
        let index_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("dc_indices"),
            size: indices_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let counter_size = 4u64;
        let counter_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("dc_counter"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        });

        let total_staging_size = counter_size + vertices_size + indices_size;
        let staging_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("dc_staging"),
            size: total_staging_size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = render_device.create_bind_group(
            "dc_bind_group",
            &dc_pipeline.layout,
            &BindGroupEntries::sequential((
                sdf_buffer.as_entire_binding(),
                vertex_buffer.as_entire_binding(),
                index_buffer.as_entire_binding(),
                counter_buffer.as_entire_binding(),
            )),
        );

        let mut encoder = render_device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
            let workgroups = (SDF_GRID_SIZE as u32) / 8;

            compute_pass.set_pipeline(qef_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroups, workgroups, workgroups);

            compute_pass.set_pipeline(quads_pipeline);
            compute_pass.dispatch_workgroups(workgroups, workgroups, workgroups);
        }

        encoder.copy_buffer_to_buffer(&counter_buffer, 0, &staging_buffer, 0, counter_size);
        encoder.copy_buffer_to_buffer(&vertex_buffer, 0, &staging_buffer, counter_size, vertices_size);
        encoder.copy_buffer_to_buffer(&index_buffer, 0, &staging_buffer, counter_size + vertices_size, indices_size);

        render_queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        let tx = sender.tx.clone();
        let staging_buffer_clone = staging_buffer.clone(); 

        buffer_slice.map_async(MapMode::Read, move |result| {
            if result.is_ok() {
                let data = staging_buffer_clone.slice(..).get_mapped_range();
                
                let count_bytes: [u8; 4] = data[0..4].try_into().unwrap();
                let index_count = u32::from_ne_bytes(count_bytes) as usize;

                let v_start = 4;
                let v_end = 4 + vertices_size as usize;
                let vertices: Vec<[f32; 3]> = bytemuck::cast_slice(&data[v_start..v_end]).to_vec();

                let i_start = v_end;
                let i_end = i_start + (index_count * 4);
                let indices: Vec<u32> = bytemuck::cast_slice(&data[i_start..i_end]).to_vec();

                drop(data);
                staging_buffer_clone.unmap();

                let _ = tx.send(MeshReadbackData { entity, vertices, indices });
            }
        });
    }
}

use bevy::asset::RenderAssetUsages;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::camera::visibility::NoFrustumCulling;

pub fn handle_compute_readbacks(
    mut commands: Commands,
    receiver: Res<ComputeReceiver>, 
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Ok(rx) = receiver.rx.try_lock() else { return; };

    for data in rx.try_iter() {
        info!("GPU Mesh Data: {} indices, {} vertices", data.indices.len(), data.vertices.len());
        
        if data.indices.is_empty() {
            continue; 
        }

        // --- TRUTH SERUM DIAGNOSTIC LOOP ---
        if !data.indices.is_empty() && (data.indices[0] as usize) < data.vertices.len() {
            info!("--- DIAGNOSTIC: FIRST TRIANGLE ---");
            for i in 0..std::cmp::min(3, data.indices.len()) {
                let idx = data.indices[i] as usize;
                let v = data.vertices[idx];
                info!("Index {}: Vertex coords [{}, {}, {}]", idx, v[0], v[1], v[2]);
            }
            info!("----------------------------------");
        }
        // ------------------------------------

        let mut normals = vec![[0.0f32; 3]; data.vertices.len()];
        
        for chunk in data.indices.chunks_exact(3) {
            let (i0, i1, i2) = (chunk[0] as usize, chunk[1] as usize, chunk[2] as usize);
            if i0 >= data.vertices.len() || i1 >= data.vertices.len() || i2 >= data.vertices.len() {
                continue;
            }
            let v0 = Vec3::from(data.vertices[i0]);
            let v1 = Vec3::from(data.vertices[i1]);
            let v2 = Vec3::from(data.vertices[i2]);
            
            let face_normal = (v1 - v0).cross(v2 - v0);
            
            normals[i0] = (Vec3::from(normals[i0]) + face_normal).to_array();
            normals[i1] = (Vec3::from(normals[i1]) + face_normal).to_array();
            normals[i2] = (Vec3::from(normals[i2]) + face_normal).to_array();
        }

        for n in normals.iter_mut() {
            let vec = Vec3::from(*n);
            *n = if vec.length_squared() > 1e-6 {
                vec.normalize().to_array()
            } else {
                [0.0, 1.0, 0.0]
            };
        }

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, data.vertices.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; data.vertices.len()]);
        mesh.insert_indices(Indices::U32(data.indices));

        let mesh_handle = meshes.add(mesh);
        
        if let Ok(mut entity_commands) = commands.get_entity(data.entity) {
            entity_commands
                .insert((Mesh3d(mesh_handle), NoFrustumCulling)) 
                .remove::<ExtractedSDF>() 
                .remove::<ComputeDispatched>(); 
                
            info!("Successfully attached GPU mesh to entity {:?}", data.entity);
        }
    }
}