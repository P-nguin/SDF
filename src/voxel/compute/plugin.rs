use bevy::prelude::*;
use bevy::render::{RenderApp, Render};
use bevy::render::extract_component::ExtractComponentPlugin;
use std::sync::{mpsc, Mutex};

use crate::voxel::compute::buffers::DualContouringPipeline;
use crate::voxel::compute::dispatch::{
    dispatch_dual_contouring, handle_compute_readbacks, 
    ComputeSender, ComputeReceiver, ExtractedSDF
};

pub struct VoxelComputePlugin;

impl Plugin for VoxelComputePlugin {
    fn build(&self, app: &mut App) {
        // Create the communication channel manually
        let (tx, rx) = mpsc::channel();

        // Give the Receiver to the Main World
        app.insert_resource(ComputeReceiver { rx: Mutex::new(rx) })
           .add_systems(Update, handle_compute_readbacks);

        // Extract all ofthe SDFs to pass over to Render World
        app.add_plugins(ExtractComponentPlugin::<ExtractedSDF>::default());

        // Give the Sender to the Render World
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.insert_resource(ComputeSender { tx });
            render_app.add_systems(Render, dispatch_dual_contouring);
        }
    }

    fn finish(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<DualContouringPipeline>();
        }
    }
}