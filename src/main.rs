mod camera;
mod qef;
mod ship;
mod voxel;

use avian3d::prelude::{GravityScale, PhysicsPlugins, RigidBody};
use bevy::prelude::*;
use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    render::{RenderPlugin, render_resource::WgpuFeatures, settings::WgpuSettings},
};
use camera::CameraPlugin;
use camera::SelectionPlugin;
use ship::{ShipPlugin, ShipTag, ShipBundle, ShipRender, ShipRenderMode};
use crate::voxel::{
    NeedsRemesh, SDF_GRID_SIZE, SDF_WORLD_SIZE, SDFChunks, SDFLayer, ShipMeshKind,
    generate_chunk_mesh_system, VoxelComputePlugin,
};

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "SDF 3D Starter".into(),
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            })
            .set(RenderPlugin {
                render_creation: WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }
                .into(),
                ..default() 
            }),
    )
    .add_plugins(WireframePlugin::default())
    .insert_resource(WireframeConfig {
        global: false,
        default_color: Color::srgb(0.0, 1.0, 0.0).into(),
        ..default()
    })
    .add_plugins(CameraPlugin)
    .add_plugins(PhysicsPlugins::default())
    .add_plugins(SelectionPlugin)
    .add_plugins(ShipPlugin)
    .add_plugins(VoxelComputePlugin) 
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (
            generate_chunk_mesh_system,
            toggle_wireframe_debug,
        ),
    );

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        PointLight {
            intensity: 2_500_000.0,
            range: 30.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(12.0, 12.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.25, 0.27, 0.29))),
    ));

    spawn_placeholder_ship(
        &mut commands,
        &mut materials,
        &asset_server,
        Transform::from_xyz(0.0, 1.25, 0.0), 
        Color::srgb(0.2, 0.6, 0.3),
    );

    spawn_placeholder_ship(
        &mut commands,
        &mut materials,
        &asset_server,
        Transform::from_xyz(10.0, 1.25, 0.0)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        Color::srgb(0.55, 0.25, 0.25),
    );
}

fn spawn_placeholder_ship(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    transform: Transform,
    exterior_color: Color,
) {
    let size = SDF_GRID_SIZE;
    let volume = (size * size * size) as usize;
    let mut exterior_distances = vec![1.0; volume];
    let mut interior_distances = vec![1.0; volume];
    let voxel_size = SDF_WORLD_SIZE / (size - 1) as f32;
    let grid_center = (size - 1) as f32 * 0.5;
    let center = Vec3::ZERO;

    let room_volumes = [
        (Vec3::new(-3.5, 0.0, 0.0), Vec3::new(2.0, 1.5, 3.0)),
        (Vec3::new(3.5, 0.0, 0.0), Vec3::new(2.0, 1.5, 3.0)),
        (center, Vec3::new(2.0, 1.0, 1.0)),
        (Vec3::new(0.0, 0.0, 2.5), Vec3::new(1.0, 1.0, 1.0)),
    ];

    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                let grid_pos = Vec3::new(x as f32, y as f32, z as f32);
                let pos = (grid_pos - Vec3::splat(grid_center)) * voxel_size;

                let hull = sdf_box(pos, Vec3::new(6.0, 2.0, 4.0));
                let mut interior_air = f32::INFINITY;

                for (room_center, room_half_extents) in room_volumes.iter() {
                    let room_dist = sdf_box(pos - *room_center, *room_half_extents);
                    interior_air = op_union(interior_air, room_dist);
                }

                let final_exterior = op_subtract(hull, interior_air);
                let final_interior = interior_air;

                let index = (z * size * size) + (y * size) + x;
                let index = index as usize;
                exterior_distances[index] = final_exterior;
                interior_distances[index] = final_interior;
            }
        }
    }

    let exterior_material = materials.add(StandardMaterial {
        base_color: exterior_color,
        cull_mode: None, // <-- Temporarily disable culling to test
        ..default()
    });
    let interior_material = materials.add(Color::srgb(0.9, 0.45, 0.15));

    let font_handle = asset_server.load("textures/font.png");
    let exterior_data_mat = materials.add(StandardMaterial {
        base_color_texture: Some(font_handle.clone()),
        base_color: Color::srgb(1.0, 0.0, 0.0),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let interior_data_mat = materials.add(StandardMaterial {
        base_color_texture: Some(font_handle.clone()),
        base_color: Color::srgb(0.0, 1.0, 0.0),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        ShipBundle {
            ship_tag: ShipTag,
            transform,
            rigidbody: RigidBody::Static,
            chunks: SDFChunks {
                layers: vec![
                    SDFLayer {
                        kind: ShipMeshKind::Exterior,
                        distances: exterior_distances,
                        material: exterior_material,
                        data_material: exterior_data_mat,
                    },
                    SDFLayer {
                        kind: ShipMeshKind::Interior,
                        distances: interior_distances,
                        material: interior_material,
                        data_material: interior_data_mat,
                    },
                ],
            },
            render: ShipRender {
                mode: ShipRenderMode::Exterior,
            },
            needs_remesh: NeedsRemesh::all(),
        },
        Visibility::Visible,
        GravityScale(0.0),
    ));
}

fn toggle_wireframe_debug(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    if keyboard.just_pressed(KeyCode::KeyZ) {
        wireframe_config.global = !wireframe_config.global;
        info!("Global wireframe mode: {}", wireframe_config.global);
    }
}

fn sdf_box(p: Vec3, extents: Vec3) -> f32 {
    let q = p.abs() - extents;
    q.max(Vec3::ZERO).length() + q.x.max(q.y.max(q.z)).min(0.0)
}

fn op_union(d1: f32, d2: f32) -> f32 {
    d1.min(d2)
}

fn op_subtract(d1: f32, d2: f32) -> f32 {
    d1.max(-d2)
}