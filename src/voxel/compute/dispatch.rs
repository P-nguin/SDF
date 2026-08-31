use bevy::{
    prelude::*,
    render::{
        mesh::allocator::MeshAllocator,
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        extract_component::ExtractComponent,
    },
};
use crate::voxel::compute::buffers::{DataRanges, DualContouringPipeline};
use crate::voxel::SDF_GRID_SIZE;

// this component needs the ExtractComponent so that the ExtractComponentPlugin can copy it into render world
// this is also why we need Clone
#[derive(Component, Clone, ExtractComponent)]
pub struct ExtractedSDF {
    pub mesh_id: AssetId<Mesh>, // we need this id to be able to find the mesh data to overwrite
    pub distances: Vec<f32>,
}

/*
    all of this code is running in the render world
    this is because of the setup in the plugin

    this allows us to access the renderdevice and renderqueue

    something else to realize is that currently only the place holder meshes are in VRAM.
    the sdf distances are still sitting in CPU memory and need to be placed into VRAM, this is what sdf_buffer is doing.
    so yeah just be aware of this split that we have RAM and then VRAM and when something is in VRAM and when we need to create a buffer to facilate the transfer into VRAM.
*/

/*  -> mesh_allocator
    in generate_mesh_system we create a max size mesh for temporary so that we can override it later in the gpu stage
    this mesh is saved into Assets<Mesh> this automatically triggered a Bevy plugin to add it to the mesh_allocator.

    When we run this code, the VRAM has already been loaded and mesh_allocator lives in VRAM. This is because of the stage it is in.
    
    We simply find the range of memory where the mesh lives and pass this slice in so that we can overwrite it
*/

/* -> render_device
    is the Rust interface for allocating new memory and objects on the GPU. It acts as the it's memory manager.
*/

/* -> render_queue
    The RenderQueue is the queue for the GPU.

    The RenderDevice only creates the resources and the command list. 
    render_device.create_command_encoder() creates a encoder and lets us encode a pipeline run into a pass.
    Once the encoder has recorded all the compute passes, you call encoder.finish() to turn it into a final CommandBuffer.

    Calling render_queue.submit(...) takes that sealed list of instructions and hands it directly to the GPU drivers. 
    This is the when the GPU spins up its cores and executes the pipeline.
*/

pub fn dispatch_dual_contouring(
    pipeline_cache: Res<PipelineCache>, // we need this to get the cached pipelines
    dc_pipeline: Res<DualContouringPipeline>, // this holds the ids for our pipelines
    render_device: Res<RenderDevice>, 
    render_queue: Res<RenderQueue>,
    mesh_allocator: Res<MeshAllocator>,
    query: Query<&ExtractedSDF>,
) {
    let qef_pipeline = pipeline_cache.get_compute_pipeline(dc_pipeline.qef_pipeline_id);
    let quads_pipeline = pipeline_cache.get_compute_pipeline(dc_pipeline.quads_pipeline_id);
    let (Some(qef_pipeline), Some(quads_pipeline)) = (qef_pipeline, quads_pipeline) else { return; };

    // for each sdf we create a independent job for it
    // TODO: as it sounds, this is not efficient and reworking it so that we can do this in bulk is much more efficient. We basically only want to call render_queue.submit once.
    for extracted_sdf in query.iter() {
        // grab out mesh id for our entity
        // this mesh is currently the blank max size one that we created in generate_mesh_system
        let mesh_id = extracted_sdf.mesh_id; 

        /*
            this bit of code below up to line 85 handles finding the actual location of the where the data lives
            we have to understand that:
                - buffers hold slabs of data
            so to find the actual location we need to do:
                - find offset
                - find slab that holds this offset
                - find buffer that holds this slab
        */
        
        // get the vertices and indexes offsets that are tied to this mesh
        let Some(vertex_slice) = mesh_allocator.mesh_vertex_slice(&mesh_id) else { continue; };
        let Some(index_slice) = mesh_allocator.mesh_index_slice(&mesh_id) else { continue; };
        
        // get the slab id for where the vertex and index data lives
        let Some(slabs) = mesh_allocator.mesh_slabs(&mesh_id) else { continue; };
        let Some(index_slab) = slabs.index_slab_id else { continue; };
        
        // get the actual buffers for where the vertex and index slabs live
        let Some(vertex_buffer) = mesh_allocator.buffer_for_slab(slabs.vertex_slab_id) else { continue; };
        let Some(index_buffer) = mesh_allocator.buffer_for_slab(index_slab) else { continue; };

        // we grab the extracted sdf distances from the cpu memory and store it into VRAM
        // TODO: we can cache this data into VRAM which allows us to skip this step if the data hasn't changed
        let sdf_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("sdf_input"),
            contents: bytemuck::cast_slice(&extracted_sdf.distances),
            usage: BufferUsages::STORAGE,
        });

        // use the offsets to specificy the location of the our meshes
        let ranges = DataRanges {
            first_vertex: vertex_slice.range.start * 8, 
            first_index: index_slice.range.start,
            _pad0: 0,
            _pad1: 0,
        };
        
        // create a buffer for the gpu to read from that holds the ranges for the mesh
        let uniform_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("dc_ranges"),
            contents: bytemuck::cast_slice(&[ranges]),
            usage: BufferUsages::UNIFORM,
        });

        // create another buffer for the gpu to hold the atomic counter
        let counter_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("dc_counter"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        // create a biindgroup using the layout defined in the buffer.rs
        let bind_group = render_device.create_bind_group(
            "dc_bind_group",
            &dc_pipeline.layout,
            &BindGroupEntries::sequential((
                sdf_buffer.as_entire_binding(),
                uniform_buffer.as_entire_binding(),
                counter_buffer.as_entire_binding(),
                vertex_buffer.as_entire_binding(),
                index_buffer.as_entire_binding(),
            )),
        );

        // encode the details of the job and add it into the render_queue
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
        render_queue.submit(std::iter::once(encoder.finish()));
    }
}