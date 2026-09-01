use bevy::prelude::*;
use bevy::render::{RenderApp, Render, RenderSystems};
use bevy::render::extract_component::ExtractComponentPlugin;

use crate::voxel::*;

pub struct VoxelComputePlugin;

impl Plugin for VoxelComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<ExtractedSDF>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else { return; };

        render_app.add_systems(
            Render,
            dispatch_dual_contouring.in_set(RenderSystems::Prepare),
        );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else { return; };
        render_app.init_resource::<DualContouringPipeline>();
    }
}

pub struct VoxelDestructionPlugin;

impl Plugin for VoxelDestructionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DestructionMode>()
            .add_systems(Update, (toggle_destruction_mode, handle_voxel_destruction));
    }
}