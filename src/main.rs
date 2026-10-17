// In src/main.rs
use avian2d::prelude::*;
use bevy::{
    prelude::*,
    window::PrimaryWindow,
    ecs::relationship::Relationship,
};

mod physics;
mod ships;
mod voxels;
mod textures;
mod navigation;
mod camera;
mod cells;
mod weapons;
mod projectiles;
mod controls;

use crate::physics::*;
use crate::ships::*;
use crate::cells::*;
use crate::textures::*;
use crate::navigation::*;
use crate::camera::*;
use crate::weapons::*;
use crate::projectiles::*;
use crate::controls::*;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Loading,
    InGame,
}

#[derive(Component)]
pub struct DummyTarget;

fn main() {
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec2::ZERO))
        .init_state::<AppState>()
        .add_plugins(ChunkRegistryPlugin)
        .add_plugins(CellRegistryPlugin)
        .add_plugins(ControlsPlugin)
        .add_plugins(NavigationPlugin)
        .add_plugins(CameraControlPlugin)
        .add_plugins(CellEffectPlugin)
        .add_plugins(WeaponPlugin)
        .add_systems(OnEnter(AppState::InGame), setup)
        .add_systems(Update, mv.run_if(in_state(AppState::InGame)))
        .add_systems(Update, crate::voxels::create_voxel_colliders.run_if(in_state(AppState::InGame)))
        .add_systems(Update, crate::projectiles::hit_detection_system.run_if(in_state(AppState::InGame)))
        .add_systems(Update, crate::voxels::update_visual_damage_system.run_if(in_state(AppState::InGame)));
        
    app.run();
}

fn setup(
    mut commands: Commands,
    chunk_registry: Res<ChunkRegistry>,
    cell_registry: Res<CellRegistry>,
) {
    commands.spawn((
        Camera2d, 
        CameraState { zoom: 1.0 }
    ));

    commands.spawn((
        Transform::from_xyz(0.0, 150.0, 0.0),
        GlobalTransform::default(),
        Sprite {
            color: Color::srgb(1.0, 0.3, 0.3),
            custom_size: Some(Vec2::new(16.0, 16.0)),
            ..default()
        },
        DummyTarget,
        crate::controls::components::Targetable,
    ));

    let blueprint1 = vec![
        vec![[" ", " "], ["-", "U"], ["-", "U"], ["-", "U"], [" ", " "]],
        vec![[" ", " "], ["-", "U"], ["T", "U"], ["-", "U"], [" ", " "]],
        vec![["-", "L"], ["-", "L"], ["*", "U"], ["-", "R"], ["-", "R"]],
        vec![[" ", " "], ["-", "D"], ["(", "U"], ["-", "D"], [" ", " "]],
        vec![[" ", " "], ["-", "D"], ["-", "D"], ["-", "D"], [" ", " "]],
    ];

    Ship::spawn_ship(
        &mut commands,
        &chunk_registry,
        &cell_registry,
        &blueprint1,
        Vec2::new(-200.0, 0.0),
    );

    let blueprint2 = vec![
        vec![[" ", " "], ["-", "U"], ["-", "U"], ["-", "U"], [" ", " "]],
        vec![[" ", " "], ["-", "U"], ["-", "U"], ["-", "U"], [" ", " "]],
        vec![["-", "L"], ["-", "L"], ["*", "U"], ["-", "R"], ["-", "R"]],
        vec![[" ", " "], ["-", "D"], ["(", "U"], ["-", "D"], [" ", " "]],
        vec![[" ", " "], ["-", "D"], ["-", "D"], ["-", "D"], [" ", " "]],
    ];

    Ship::spawn_ship(
        &mut commands,
        &chunk_registry,
        &cell_registry,
        &blueprint2,
        Vec2::new(200.0, 0.0),
    );
}

fn mv(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_ship: Query<(Entity, &Transform), (With<Ship>, With<Selected>)>,
) {
    // Only trigger once the exact moment 'M' is pressed
    if !keyboard_input.just_pressed(KeyCode::KeyM) {
        return;
    }
    let Ok(window) = q_window.single() else { return };
    let Ok((camera, camera_transform)) = q_camera.single() else { return };

    // Check if the cursor is currently inside the window
    if let Some(cursor_pos) = window.cursor_position() {
        // Convert the on-screen pixel coordinates to world space coordinates
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            
            // Loop through all spawned ships and command them to move
            for (ship_entity, transform) in q_ship.iter() {
                let current_pos = transform.translation.truncate();
                let heading = if (world_pos - current_pos).length_squared() > 1.0 {
                    (world_pos - current_pos).normalize()
                } else {
                    (transform.rotation * Vec3::Y).truncate().normalize()
                };

                let waypoint = crate::navigation::Waypoint {
                    position: world_pos,
                    target_speed: 0.0,
                    target_heading: heading,
                    radius: 12.0, 
                };

                let mut plan = crate::navigation::NavigationPlan::default();
                plan.waypoints.push_back(waypoint);

                commands.entity(ship_entity).insert(plan);
            }
        }
    }
}

