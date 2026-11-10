//! Exposes a plugin and CLI to congigure a small number of settings for the game.
use bevy::{
    app::{App, Plugin, Startup, Update},
    camera::{Camera2d, Camera3d, Hdr},
    core_pipeline::tonemapping::Tonemapping,
    ecs::{
        entity::Entity,
        query::{With, Without},
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, ResMut},
    },
    log::error,
    prelude::Deref,
    state::{condition::in_state, state::OnEnter},
};
use leafwing_input_manager::prelude::InputManagerPlugin;
use thetawave_interface::{
    game::options::GameOptions,
    input::{InputsResource, MenuAction},
    states,
};

pub(super) mod display;
mod input;

use input::get_input_bindings;
use std::default::Default;
#[cfg(not(feature = "playtest"))]
use std::env::current_dir;
#[cfg(not(feature = "playtest"))]
use std::fs::{DirBuilder, File};
#[cfg(not(feature = "playtest"))]
use std::io::Write;
use std::path::PathBuf;

use self::display::{set_window_icon, toggle_fullscreen_system, toggle_zoom_system};

use self::input::spawn_menu_explorer_system;

// Bevy 0.20 port: upstream's optional `cli` feature (argh-based CLI
// parsing) was removed along with the feature itself; these options
// now always take their defaults.
#[derive(Default, Debug, PartialEq, Eq)]
/// Options used to start Thetawave. As many of these as possible are inferred/have "sensible"
/// defaults.
pub struct GameInitCLIOptions {
    /// the directory that is used for `bevy::asset::AssetPlugin`. This is generally
    /// 'EXECUTABLE_DIR/assets/' or 'CARGO_MANIFEST_DIR/assets'.
    pub assets_dir: Option<PathBuf>,
    /// whether to use instructions, serial port IO, etc. specific to deploying on an arcade
    /// machine. Unused: the serial-port arcade-cabinet support was removed in
    /// the Bevy 0.20 port (`bevy_serialport` has no compatible release).
    pub arcade: bool,
}
impl GameInitCLIOptions {
    pub fn from_environ_on_supported_platforms_with_default_fallback() -> Self {
        Default::default()
    }
}

fn apply_game_options_system(
    mut commands: Commands,
    mut game_options: ResMut<GameOptions>,
    camera_2d_query: Query<(Entity, Option<&Hdr>), (With<Camera2d>, Without<Camera3d>)>,
    mut tonemapping_2d_query: Query<(&mut Tonemapping,), (With<Camera2d>, Without<Camera3d>)>,
    camera_3d_query: Query<(Entity, Option<&Hdr>), (With<Camera3d>, Without<Camera2d>)>,
    mut tonemapping_3d_query: Query<(&mut Tonemapping,), (With<Camera3d>, Without<Camera2d>)>,
) {
    if let (Ok((camera_2d_entity, hdr_2d)), Ok((camera_3d_entity, hdr_3d))) =
        (camera_2d_query.single(), camera_3d_query.single())
    {
        // Bevy 0.20 port: `Camera::hdr` became the `Hdr` marker component.
        if game_options.bloom_enabled {
            if hdr_2d.is_none() {
                commands.entity(camera_2d_entity).insert(Hdr);
            }
            if hdr_3d.is_none() {
                commands.entity(camera_3d_entity).insert(Hdr);
            }
        } else {
            commands.entity(camera_2d_entity).remove::<Hdr>();
            commands.entity(camera_3d_entity).remove::<Hdr>();
        }

        let tonemapping = if game_options.bloom_enabled {
            Tonemapping::TonyMcMapface
        } else {
            Tonemapping::None
        };

        if let Ok((mut tonemapping_2d,)) = tonemapping_2d_query.single_mut() {
            *tonemapping_2d = tonemapping;
        }
        if let Ok((mut tonemapping_3d,)) = tonemapping_3d_query.single_mut() {
            *tonemapping_3d = tonemapping;
            if !game_options.bloom_enabled {
                game_options.bloom_intensity = 0.0;
            }
        }
    } else {
        error!("Failed to get singleton 2d and 3d cameras to apply game opts");
    }
}

/// Whether we are playing on an arcade machine. This affects some different UI elements.
/// Generally this will be set at app startup either inferred or explicitly provided as a game
/// startup parameter, and should probably not be mutated during the game.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct PlayingOnArcadeResource(bool);

/// Includes systems to change configuration settings about the game.
#[derive(Default)]
pub(super) struct OptionsPlugin {
    pub arcade: bool,
}

impl Plugin for OptionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<MenuAction>::default());

        app.insert_resource(InputsResource::from(get_input_bindings()));
        app.insert_resource(PlayingOnArcadeResource(self.arcade));
        app.insert_resource(GameOptions::default());

        app.add_systems(Startup, spawn_menu_explorer_system);

        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Startup, set_window_icon);

        app.add_systems(Update, toggle_fullscreen_system);

        app.add_systems(
            Update,
            toggle_zoom_system.run_if(in_state(states::AppStates::Game)),
        );

        app.add_systems(
            OnEnter(states::AppStates::MainMenu),
            apply_game_options_system,
        );
    }
}

/// Creates config file in config directory from config file in this directory
#[cfg(not(feature = "playtest"))]
macro_rules! confgen {
    ( $($filename:expr),* ) => {
        {
            let conf_dir = current_dir().unwrap().join("config");
            if !conf_dir.is_dir() {
                DirBuilder::new()
                    .create(conf_dir.clone())
                    .expect("Confgen failed: could not create config dir.");
            }

            $({
                let default = include_bytes!($filename);
                let file_path = conf_dir.join($filename);
                let mut file = File::create(file_path)
                    .expect(concat!("Confgen failed: could not create config file ", $filename, "."));
                file.write_all(default)
                    .expect(concat!("Confgen failed: could not write config file ", $filename, "."));
            })*
        }
    }
}

/// Generates the display config file
#[cfg(not(feature = "playtest"))]
pub(super) fn generate_config_files() {
    confgen!("display.ron");
    confgen!("input.ron");
}
