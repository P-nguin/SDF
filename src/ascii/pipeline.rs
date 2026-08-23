use bevy::{
    core_pipeline::{
        FullscreenShader, 
        prepass::ViewPrepassTextures,
    },
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::{
            binding_types::{sampler, texture_2d},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
        render_asset::RenderAssets,
        texture::GpuImage,
    },
};

#[derive(Component, Clone, ExtractComponent)]
pub struct AsciiSettings {
    pub enabled: bool,
    pub font_handle: Handle<Image>,
}

#[derive(Resource)]
pub struct AsciiPipeline {
    pub layout: BindGroupLayout,
    pub pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for AsciiPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout_entries = BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: false }),
                texture_2d(TextureSampleType::Depth), 
                sampler(SamplerBindingType::NonFiltering),
                texture_2d(TextureSampleType::Float { filterable: true }), 
            ),
        );

        let layout = render_device.create_bind_group_layout(
            "ascii_bind_group_layout",
            &layout_entries,
        );

        let shader = world.resource::<AssetServer>().load("shaders/ascii.wgsl");
        let vertex_state = world.resource::<FullscreenShader>().to_vertex_state();

        let pipeline_id = world.resource_mut::<PipelineCache>().queue_render_pipeline(
            RenderPipelineDescriptor {
                label: Some("ascii_pipeline".into()),
                layout: vec![BindGroupLayoutDescriptor {
                    label: "ascii_bind_group_layout".into(),
                    entries: layout_entries.to_vec(),
                }],
                vertex: vertex_state,
                fragment: Some(FragmentState {
                    shader,
                    shader_defs: vec![],
                    entry_point: Some("fragment".into()), 
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::Rgba8UnormSrgb, 
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                zero_initialize_workgroup_memory: true,
                immediate_size: 0, 
            },
        );

        Self { layout, pipeline_id }
    }
}

pub fn ascii_post_process(
    mut render_context: RenderContext,
    view_query: Query<(&ViewTarget, &ViewPrepassTextures, &AsciiSettings)>,
    pipeline: Res<AsciiPipeline>,
    pipeline_cache: Res<PipelineCache>,
    gpu_images: Res<RenderAssets<GpuImage>>,
) {
    let Some(render_pipeline) = pipeline_cache.get_render_pipeline(pipeline.pipeline_id) else {
        return;
    };

    for (view_target, prepass_textures, settings) in view_query.iter() {
        if !settings.enabled {
            continue;
        }

        let Some(depth_view) = prepass_textures.depth_view() else {
            continue;
        };

        let Some(gpu_font) = gpu_images.get(&settings.font_handle) else {
            continue; 
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "ascii_bind_group",
            &pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                depth_view,
                &render_context.render_device().create_sampler(&SamplerDescriptor::default()),
                &gpu_font.texture_view,
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("ascii_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
                depth_slice: None, 
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None, 
        });

        render_pass.set_render_pipeline(render_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
