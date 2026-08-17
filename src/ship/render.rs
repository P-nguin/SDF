use crate::player::Selected;
use crate::ship::{ShipRender, ShipRenderMode};
use crate::voxel::{GeneratedShipMesh, ShipMeshKind};
use bevy::prelude::*;

// toggle ship render_mode when pressing v
pub fn toggle_ship_render_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // 1. Only fetch the ship that currently has the `Selected` component
    mut selected_ships: Query<(Entity, &mut ShipRender), With<Selected>>,
    // 2. Fetch all meshes, but we will filter them using the entity ID
    mut meshes: Query<(
        &GeneratedShipMesh,
        &MeshMaterial3d<StandardMaterial>,
        &mut Visibility,
    )>,
) {
    if !keys.just_pressed(KeyCode::KeyV) {
        return;
    }

    let Ok((selected_entity, mut ship_render)) = selected_ships.single_mut() else {
        return;
    };

    // Toggle the render state for this specific ship
    ship_render.mode = match ship_render.mode {
        ShipRenderMode::Exterior => ShipRenderMode::Interior,
        ShipRenderMode::Interior => ShipRenderMode::Both,
        ShipRenderMode::Both => ShipRenderMode::Exterior,
    };

    // Apply the visual changes
    for (mesh_info, material_handle, mut visibility) in meshes.iter_mut() {
        // IMPORTANT: Skip this mesh if it does not belong to the selected ship
        if mesh_info.ship != selected_entity {
            continue;
        }

        let should_show = match ship_render.mode {
            ShipRenderMode::Exterior => mesh_info.kind == ShipMeshKind::Exterior,
            ShipRenderMode::Interior => mesh_info.kind == ShipMeshKind::Interior,
            ShipRenderMode::Both => true,
        };

        *visibility = if should_show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        if let Some(mut material) = materials.get_mut(&material_handle.0) {
            if ship_render.mode == ShipRenderMode::Both {
                material.base_color.set_alpha(0.3);
                material.alpha_mode = AlphaMode::Blend;
            } else {
                material.base_color.set_alpha(1.0);
                material.alpha_mode = AlphaMode::Opaque;
            }
        }
    }
}
