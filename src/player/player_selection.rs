use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::player::player_set_focus::focus_on_selected_ship;
use crate::voxel::GeneratedShipMesh;

pub struct PlayerSelectionPlugin;

impl Plugin for PlayerSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (select_ship_with_mouse, focus_on_selected_ship));
    }
}

#[derive(Component, Default, Clone, Copy)]
pub struct Selected;

fn select_ship_with_mouse(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    spatial_query: SpatialQuery,
    generated_ship_meshes: Query<&GeneratedShipMesh>,
    selected_ships: Query<Entity, With<Selected>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let mut ship_mesh_filter = |entity| generated_ship_meshes.get(entity).is_ok();
    let hit = spatial_query.cast_ray_predicate(
        ray.origin,
        ray.direction,
        f32::MAX,
        true,
        &SpatialQueryFilter::default(),
        &mut ship_mesh_filter,
    );

    for selected_entity in &selected_ships {
        commands.entity(selected_entity).remove::<Selected>();
    }

    let Some(hit) = hit else {
        return;
    };
    let hit_entity = hit.entity;
    let Ok(mesh_part) = generated_ship_meshes.get(hit_entity) else {
        return;
    };

    commands.entity(mesh_part.ship).insert(Selected);
    info!("Selected ship: {:?}", mesh_part.ship);
}
