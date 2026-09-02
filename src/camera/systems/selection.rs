use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::ship::ShipTag; // <-- Import the parent tag
use crate::camera::Selected;

pub fn select_ship_with_mouse(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    spatial_query: SpatialQuery,
    // Change the query to look for the parent ShipTag
    ships: Query<(), With<ShipTag>>, 
    selected_ships: Query<Entity, With<Selected>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };
    let Ok((camera, camera_transform)) = cameras.single() else { return; };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else { return; };

    // Filter against the parent ShipTag instead of the child mesh
    let mut ship_filter = |entity| ships.get(entity).is_ok();

    let hit = spatial_query.cast_ray_predicate(
        ray.origin,
        ray.direction,
        f32::MAX,
        true,
        &SpatialQueryFilter::default(),
        &mut ship_filter,
    );

    // Clear previous selection
    for selected_entity in &selected_ships {
        commands.entity(selected_entity).remove::<Selected>();
    }

    let Some(hit) = hit else { return; };

    // The hit entity is the parent, so we can insert Selected directly
    commands.entity(hit.entity).insert(Selected);
    info!("Selected ship: {:?}", hit.entity);
}