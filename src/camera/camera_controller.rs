use bevy::{
    input::mouse::{MouseButton, MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, pan_orbit_camera);
    }
}

// ==========================================
// COMPONENTS
// ==========================================

#[derive(Bundle, Default)]
pub struct PanOrbitCameraBundle {
    pub camera: (Camera3d, Transform),
    pub state: PanOrbitState,
    pub settings: PanOrbitSettings,
}

#[derive(Component)]
pub struct PanOrbitState {
    pub center: Vec3,
    pub radius: f32,
    pub upside_down: bool,
    pub pitch: f32,
    pub yaw: f32,
    pub focus: Option<Entity>,
}

#[derive(Component)]
pub struct PanOrbitSettings {
    /// World units per pixel of mouse motion
    pub pan_sensitivity: f32,
    /// World units per second for WASD movement
    pub keyboard_move_speed: f32,
    /// Radians per pixel of mouse motion
    pub orbit_sensitivity: f32,
    /// Exponent per pixel of mouse motion
    pub zoom_sensitivity: f32,
    /// Key to hold for panning
    pub pan_key: Option<KeyCode>,
    /// Mouse button to hold for panning
    pub pan_mouse_button: Option<MouseButton>,
    /// Key to hold for orbiting
    pub orbit_key: Option<KeyCode>,
    /// Mouse button to hold for orbiting
    pub orbit_mouse_button: Option<MouseButton>,
    /// Key to hold for zooming
    pub zoom_key: Option<KeyCode>,
    /// Mouse button to hold for zooming
    pub zoom_mouse_button: Option<MouseButton>,
    /// What action is bound to the scroll wheel?
    pub scroll_action: Option<PanOrbitAction>,
    /// For devices with a notched scroll wheel, like desktop mice
    pub scroll_line_sensitivity: f32,
    /// For devices with smooth scrolling, like touchpads
    pub scroll_pixel_sensitivity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanOrbitAction {
    Pan,
    Orbit,
    Zoom,
}

impl Default for PanOrbitState {
    fn default() -> Self {
        PanOrbitState {
            center: Vec3::ZERO,
            radius: 1.0,
            upside_down: false,
            pitch: 0.0,
            yaw: 0.0,
            focus: None,
        }
    }
}

impl Default for PanOrbitSettings {
    fn default() -> Self {
        PanOrbitSettings {
            pan_sensitivity: 0.001, // 1000 pixels per world unit
            keyboard_move_speed: 8.0,
            orbit_sensitivity: 0.1f32.to_radians(), // 0.1 degree per pixel
            zoom_sensitivity: 0.01,
            pan_key: Some(KeyCode::ControlLeft),
            pan_mouse_button: None,
            orbit_key: None,
            orbit_mouse_button: Some(MouseButton::Right),
            zoom_key: Some(KeyCode::ShiftLeft),
            zoom_mouse_button: None,
            scroll_action: Some(PanOrbitAction::Zoom),
            scroll_line_sensitivity: 16.0, // 1 "line" == 16 "pixels of motion"
            scroll_pixel_sensitivity: 1.0,
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = PanOrbitCameraBundle::default();
    // Position our camera using our component,
    // not Transform (it would get overwritten)
    camera.state.center = Vec3::ZERO;
    camera.state.radius = 10.0;
    camera.state.pitch = 25.0f32.to_radians();
    camera.state.yaw = -30.0f32.to_radians();
    commands.spawn(camera);
}

fn pan_orbit_camera(
    time: Res<Time>,
    kbd: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut evr_motion: MessageReader<MouseMotion>,
    mut evr_scroll: MessageReader<MouseWheel>,
    mut q_camera: Query<(&PanOrbitSettings, &mut PanOrbitState, &mut Transform)>,
    target_query: Query<&Transform, Without<PanOrbitState>>,
) {
    // First, accumulate the total amount of
    // mouse motion and scroll, from all pending events:
    let total_motion: Vec2 = evr_motion.read().map(|ev| ev.delta).sum();
    let mut total_scroll_lines = Vec2::ZERO;
    let mut total_scroll_pixels = Vec2::ZERO;
    for ev in evr_scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                total_scroll_lines.x += ev.x;
                total_scroll_lines.y -= ev.y;
            }
            MouseScrollUnit::Pixel => {
                total_scroll_pixels.x += ev.x;
                total_scroll_pixels.y -= ev.y;
            }
        }
    }

    for (settings, mut state, mut transform) in &mut q_camera {
        // Check how much of each thing we need to apply.
        // Accumulate values from motion and scroll,
        // based on our configuration settings.

        let mut total_pan = Vec2::ZERO;
        if action_pressed(
            &kbd,
            &mouse_buttons,
            settings.pan_key,
            settings.pan_mouse_button,
        ) {
            total_pan -= total_motion * settings.pan_sensitivity;
        }
        if settings.scroll_action == Some(PanOrbitAction::Pan) {
            total_pan -=
                total_scroll_lines * settings.scroll_line_sensitivity * settings.pan_sensitivity;
            total_pan -=
                total_scroll_pixels * settings.scroll_pixel_sensitivity * settings.pan_sensitivity;
        }

        let mut total_orbit = Vec2::ZERO;
        if action_pressed(
            &kbd,
            &mouse_buttons,
            settings.orbit_key,
            settings.orbit_mouse_button,
        ) {
            total_orbit -= total_motion * settings.orbit_sensitivity;
        }
        if settings.scroll_action == Some(PanOrbitAction::Orbit) {
            total_orbit -=
                total_scroll_lines * settings.scroll_line_sensitivity * settings.orbit_sensitivity;
            total_orbit -= total_scroll_pixels
                * settings.scroll_pixel_sensitivity
                * settings.orbit_sensitivity;
        }

        let mut total_zoom = Vec2::ZERO;
        if action_pressed(
            &kbd,
            &mouse_buttons,
            settings.zoom_key,
            settings.zoom_mouse_button,
        ) {
            total_zoom -= total_motion * settings.zoom_sensitivity;
        }
        if settings.scroll_action == Some(PanOrbitAction::Zoom) {
            total_zoom -=
                total_scroll_lines * settings.scroll_line_sensitivity * settings.zoom_sensitivity;
            total_zoom -=
                total_scroll_pixels * settings.scroll_pixel_sensitivity * settings.zoom_sensitivity;
        }

        // Upon starting a new orbit maneuver (key is just pressed),
        // check if we are starting it upside-down
        if action_just_pressed(
            &kbd,
            &mouse_buttons,
            settings.orbit_key,
            settings.orbit_mouse_button,
        ) {
            state.upside_down = state.pitch < -FRAC_PI_2 || state.pitch > FRAC_PI_2;
        }

        // If we are upside down, reverse the X orbiting
        if state.upside_down {
            total_orbit.x = -total_orbit.x;
        }

        // Now we can actually do the things!

        let mut any = false;

        // If we have focus target, we need to move track it with the camera transform
        if let Some(target_entity) = state.focus {
            if let Ok(target_transform) = target_query.get(target_entity) {
                state.center = target_transform.translation;
                any = true;
            } else {
                // If the ship was destroyed or despawned, break the lock
                state.focus = None;
            }
        }

        // To ZOOM, we need to multiply our radius.
        if total_zoom != Vec2::ZERO {
            any = true;
            // in order for zoom to feel intuitive,
            // everything needs to be exponential
            // (done via multiplication)
            // not linear
            // (done via addition)

            // so we compute the exponential of our
            // accumulated value and multiply by that
            state.radius *= (-total_zoom.y).exp();
        }

        // To ORBIT, we change our pitch and yaw values
        if total_orbit != Vec2::ZERO {
            any = true;
            state.yaw += total_orbit.x;
            state.pitch += total_orbit.y;
            // wrap around, to stay between +- 180 degrees
            if state.yaw > PI {
                state.yaw -= TAU; // 2 * PI
            }
            if state.yaw < -PI {
                state.yaw += TAU; // 2 * PI
            }
            if state.pitch > PI {
                state.pitch -= TAU; // 2 * PI
            }
            if state.pitch < -PI {
                state.pitch += TAU; // 2 * PI
            }
        }

        // To PAN, we can get the UP and RIGHT direction
        // vectors from the camera's transform, and use
        // them to move the center point. Multiply by the
        // radius to make the pan adapt to the current zoom.
        if total_pan != Vec2::ZERO {
            any = true;
            let radius = state.radius;
            state.center += transform.right() * total_pan.x * radius;
            state.center += transform.up() * total_pan.y * radius;
        }

        let mut keyboard_move = Vec3::ZERO;
        let yaw_rotation = Quat::from_rotation_y(state.yaw);
        let forward = yaw_rotation * Vec3::NEG_Z;
        let right = yaw_rotation * Vec3::X;

        if kbd.pressed(KeyCode::KeyW) {
            keyboard_move += forward;
        }
        if kbd.pressed(KeyCode::KeyS) {
            keyboard_move -= forward;
        }
        if kbd.pressed(KeyCode::KeyA) {
            keyboard_move -= right;
        }
        if kbd.pressed(KeyCode::KeyD) {
            keyboard_move += right;
        }

        if keyboard_move != Vec3::ZERO {
            any = true;
            state.focus = None;
            state.center += keyboard_move.normalize_or_zero()
                * settings.keyboard_move_speed
                * time.delta_secs();
        }

        // Finally, compute the new camera transform.
        // (if we changed anything, or if the pan-orbit
        // controller was just added and thus we are running
        // for the first time and need to initialize)
        if any || state.is_added() {
            // YXZ Euler Rotation performs yaw/pitch/roll.
            transform.rotation = Quat::from_euler(EulerRot::YXZ, state.yaw, state.pitch, 0.0);
            // To position the camera, get the backward direction vector
            // and place the camera at the desired radius from the center.
            transform.translation = state.center + transform.back() * state.radius;
        }
    }
}

fn action_pressed(
    kbd: &ButtonInput<KeyCode>,
    mouse_buttons: &ButtonInput<MouseButton>,
    key: Option<KeyCode>,
    mouse_button: Option<MouseButton>,
) -> bool {
    key.map(|key| kbd.pressed(key)).unwrap_or(false)
        || mouse_button
            .map(|button| mouse_buttons.pressed(button))
            .unwrap_or(false)
}

fn action_just_pressed(
    kbd: &ButtonInput<KeyCode>,
    mouse_buttons: &ButtonInput<MouseButton>,
    key: Option<KeyCode>,
    mouse_button: Option<MouseButton>,
) -> bool {
    key.map(|key| kbd.just_pressed(key)).unwrap_or(false)
        || mouse_button
            .map(|button| mouse_buttons.just_pressed(button))
            .unwrap_or(false)
}
