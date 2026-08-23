use bevy::{
    core_pipeline::{Core3d, Core3dSystems},
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin,
        RenderApp,
    },
};
use crate::ascii::pipeline::{ascii_post_process, AsciiPipeline, AsciiSettings};

pub struct AsciiRenderPlugin;

impl Plugin for AsciiRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<AsciiSettings>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(
                Core3d,
                // Order it to run after Bevy finishes the main 3D pass and before it tries to upscale to avoid flickering buffers
                ascii_post_process
                    .after(Core3dSystems::PostProcess)
                    .before(bevy::core_pipeline::upscaling::upscaling),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        }; 
        
        render_app.init_resource::<AsciiPipeline>();
    }
}
