use bevy::{
    prelude::*,
    render::render_resource::*,
    render::renderer::*,
    render::render_resource::{binding_types::*, ShaderType},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
pub struct DataRanges {
    pub first_vertex: u32,
    pub first_index: u32,
    pub _pad0: u32,
    pub _pad1: u32,
}

// we have one pipeline for each pass
#[derive(Resource)]
pub struct DualContouringPipeline {
    // dictates the memory layout of the input for each pipeline
    // the two pipelines need the same input so we can just define one layout for both
    pub layout: BindGroupLayout,
    pub qef_pipeline_id: CachedComputePipelineId,
    pub quads_pipeline_id: CachedComputePipelineId,
}

// initalize the pipeline
impl FromWorld for DualContouringPipeline {
    fn from_world(world: &mut World) -> Self {
        // get the render world
        let render_device = world.resource::<RenderDevice>();
        
        // define the layout memory for the shaders
        let layout_entries = BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer_read_only::<Vec<f32>>(false), // 0: SDF 
                uniform_buffer::<DataRanges>(false),         // 1: MeshAllocator Offsets (16-byte aligned)
                storage_buffer::<u32>(false),                // 2: Atomic Index Counter
                storage_buffer::<Vec<f32>>(false),           // 3: Global Vertices (flat f32)
                storage_buffer::<Vec<u32>>(false),           // 4: Global Indices
            ),
        );

        // actually create the memory binding with the layout
        // then load and compile the shader code
        // grab the pipline cache so that after compiling and creating the bind groups we can cache it for the next time we need it
        //      this is because loading + compiling + creating bind groups is a expensive process
        let layout = render_device.create_bind_group_layout("dc_bind_group_layout", &layout_entries);
        let shader = world.resource::<AssetServer>().load("shaders/dual_contouring.wgsl");
        let pipeline_cache = world.resource_mut::<PipelineCache>();

        // notice that layout_desc is a vector with one BindGroupLayoutDescriptor
        // this means that all of this is getting grouped into memory slot 0
        // this BindGroupLayoutDescriptor holds this label and then the memory layout
        let layout_desc = vec![BindGroupLayoutDescriptor {
            label: "dc_bind_group_layout".into(), // this is just for debugging
            entries: layout_entries.to_vec(), // holds the actual memory layout
        }];

        // Create and cache the pipeline for the pass 1
        let qef_pipeline_id = pipeline_cache.queue_compute_pipeline(
            ComputePipelineDescriptor {
                label: Some("dc_qef_pipeline".into()),
                layout: layout_desc.clone(),
                shader: shader.clone(),
                shader_defs: vec![],
                entry_point: Some("solve_qef".into()), 
                zero_initialize_workgroup_memory: true,
                immediate_size: 0, 
            },
        );
        
        // Create and cache the pipeline for the pass 2
        let quads_pipeline_id = pipeline_cache.queue_compute_pipeline(
            ComputePipelineDescriptor {
                label: Some("dc_quads_pipeline".into()),
                layout: layout_desc,
                shader,
                shader_defs: vec![],
                entry_point: Some("generate_quads".into()), 
                zero_initialize_workgroup_memory: true,
                immediate_size: 0, 
            },
        );

        // return the complete DualContouringPipeline struct
        Self { layout, qef_pipeline_id, quads_pipeline_id }
    }
}