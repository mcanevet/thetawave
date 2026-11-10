//! Exposes a plugin that sets up the 2D/3D perspective/camera and shakes the camera when an event
//! is emitted.
use crate::game;
use bevy::app::{App, Plugin, Startup, Update};
use bevy::camera::{
    Camera, Camera2d, Camera3d, ClearColorConfig, Hdr, PerspectiveProjection, Projection,
};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::ecs::system::{Commands, Res};
use bevy::math::Vec3;
use bevy::post_process::bloom::{Bloom, BloomPrefilter};
use bevy::transform::components::Transform;
use bevy::utils::default;

use self::screen_shake::{
    add_trauma_system, screen_shake_on_player_damage_system, screen_shake_system,
};
use thetawave_interface::camera::ScreenShakeEvent;

mod screen_shake;

pub(super) struct CameraPlugin;

/// Sets up a 2d perspective/camera of the 3d world. When this plugin is enabled, one can send
/// `thetawave_interface::camera::ScreenShakeEvent` to jolt the screen, for example, when a player
/// takes damage.
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ScreenShakeEvent>();
        app.add_systems(Startup, setup_cameras_system);
        app.add_systems(
            Update,
            (
                screen_shake_system,
                add_trauma_system,
                screen_shake_on_player_damage_system,
            ),
        );
    }
}

/// Setup 2D camera for sprites used in gameplay
/// Setup 3D camera for rendering 3D backgrounds
fn setup_cameras_system(
    mut commands: Commands,
    game_parameters: Res<game::GameParametersResource>,
) {
    // setup cameras
    // 2d camera for sprites
    // Bevy 0.20 port: camera bundles were removed; required components
    // supply defaults, HDR is now the `Hdr` marker component, and bloom
    // moved to `bevy::post_process::bloom::Bloom`.
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Hdr,
        Tonemapping::TonyMcMapface,
        Transform::from_xyz(0.0, 0.0, game_parameters.camera_z),
        Bloom {
            prefilter: BloomPrefilter {
                threshold: 1.0,
                threshold_softness: 0.2,
            },
            ..Bloom::OLD_SCHOOL
        },
        screen_shake::ScreenShakeComponent {
            trauma: 0.0,
            trauma_decay: 1.,
            shake_intensity: Vec3 {
                x: 60.,
                y: 60.,
                z: 0.1,
            },
        },
    ));

    // 3d camera for background objects
    commands.spawn((
        // Bevy 0.20 port: `Camera3d` gained fields; use `Default`.
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        Hdr,
        Tonemapping::TonyMcMapface,
        Transform::from_xyz(0.0, 0.0, game_parameters.camera_z).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            far: 10000.0,
            ..Default::default()
        }),
        Bloom::default(),
    ));
}
