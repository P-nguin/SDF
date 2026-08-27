use bevy::{
    prelude::*,
    render::render_resource::*,
    render::renderer::*,
    render::render_resource::binding_types::*,
};

#[derive(Resource)]
pub struct DualContouringPipeline {
    pub layout: BindGroupLayout,
    pub qef_pipeline_id: CachedComputePipelineId,
    pub quads_pipeline_id: CachedComputePipelineId,
}

impl FromWorld for DualContouringPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        
        // Define the 4 memory slots the GPU will use:
        // Slot 0: Read-only SDF array
        // Slot 1: Read/Write Vertex buffer
        // Slot 2: Read/Write Index buffer
        // Slot 3: Read/Write Atomic counter
        let layout_entries = BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer_read_only::<Vec<f32>>(false),
                storage_buffer::<Vec<[f32; 3]>>(false),
                storage_buffer::<Vec<u32>>(false),
                storage_buffer::<u32>(false), 
            ),
        );

        let layout = render_device.create_bind_group_layout(
            "dc_bind_group_layout",
            &layout_entries,
        );

        let shader = world.resource::<AssetServer>().load("shaders/dual_contouring.wgsl");
        let pipeline_cache = world.resource_mut::<PipelineCache>();

        let layout_desc = vec![BindGroupLayoutDescriptor {
            label: "dc_bind_group_layout".into(),
            entries: layout_entries.to_vec(),
        }];

        // Queue Pass 1: Vertex Generation
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

        // Queue Pass 2: Index/Quad Stitching
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

        Self { 
            layout, 
            qef_pipeline_id,
            quads_pipeline_id
        }
    }
}