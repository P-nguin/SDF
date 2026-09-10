// In src/main.rs
use avian2d::prelude::*;
use bevy::{
    prelude::*,
    window::PrimaryWindow,
};

mod ships;
mod voxels;
mod textures;
mod navigation;

use crate::ships::*;
use crate::textures::*;
use crate::navigation::*;

fn main() {
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec2::ZERO))        
        .add_plugins(ChunkRegistryPlugin)
        .add_plugins(MoveToPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, mv);
        
    app.run();
}

fn setup(
    mut commands: Commands,
    registry: Res<ChunkRegistry>, 
) {
    commands.spawn(Camera2d);

    let blueprint = [
        " --- ",
        " --- ",
        "-----",
        " --- ",
        " --- ",
    ];

    Ship::spawn_ship(
        &mut commands,
        &registry,
        &blueprint,
        Vec2::new(0.0, 5.0),
    );
}

fn mv(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_ship: Query<Entity, With<Ship>>,
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
            for ship_entity in q_ship.iter() {
                commands.entity(ship_entity).insert(MoveTo {
                    target: world_pos,
                    rad: 12.0, // Arrival radius 
                });
            }
        }
    }
}