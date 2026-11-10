#![doc = include_str!("../README.md")]
// The playtest feature links the whole harness into the game binary,
// which on macOS arm64 crosses ld's 16MB compact-unwind encoding
// limit (performance-only note, unavoidable for a linked-in-harness
// debug build).
#![cfg_attr(feature = "playtest", allow(linker_messages))]
// Vendored upstream style, kept as-is for the port (fork carries the
// same decision): Bevy systems with many query/event parameters and
// wide tupled queries are idiomatic in 0.14-era thetawave; refactoring
// 15-argument systems would be invasive churn with no behavioral gain.
#![allow(clippy::type_complexity, clippy::too_many_arguments)]
// Upstream enum shapes (Gain* consumable effects, timer-heavy
// TutorialLesson/LevelPhaseType variants) are data-driven from RON;
// boxing or renaming would churn serialized formats for style only.
#![allow(clippy::enum_variant_names, clippy::large_enum_variant)]
use bevy::app::PluginGroupBuilder;
use bevy::light::{AmbientLight, GlobalAmbientLight};
use bevy::prelude::{
    App, AppExtStates, AssetPlugin, ClearColor, Color, Commands, ImagePlugin, IntoScheduleConfigs,
    OnEnter, PluginGroup, Query, SystemSet, Vec2, Window, WindowPlugin, With,
};
use bevy_kira_audio::prelude::AudioPlugin;

use crate::options::display::DisplayConfig;
use bevy_rapier2d::plugin::context::DefaultRapierContext;
use bevy_rapier2d::prelude::{
    NoUserData, RapierConfiguration, RapierDebugRenderPlugin, RapierPhysicsPlugin, TimestepMode,
};
#[cfg(all(not(target_arch = "wasm32"), not(feature = "playtest")))]
use options::generate_config_files;
use options::GameInitCLIOptions;
use thetawave_interface::states::{AppStates, GameStates};

/// Used by a physics engine to translate physics calculations to graphics
const PHYSICS_PIXELS_PER_METER: f32 = 10.0;

mod animation;
mod arena;
mod audio;
mod background;
mod camera;
mod collision;
mod game;
mod health;
mod loot;
mod options;
mod player;
mod run;
mod scanner;
mod spawnable;
mod states;
mod tools;
mod ui;
mod weapon;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum GameEnterSet {
    Initialize,
    BuildLevel,
    SpawnPlayer,
    BuildUi,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum GameUpdateSet {
    Enter,
    Level,
    Spawn,
    NextLevel,
    UpdateUi,
    Movement,
    Abilities,
    SetTargetBehavior, // TODO: replace with more general set
    ExecuteBehavior,
    ContactCollision,
    IntersectionCollision,
    ApplyDisconnectedBehaviors,
    ChangeState,
    Cleanup,
}

#[cfg(not(target_arch = "wasm32"))]
fn get_display_config() -> DisplayConfig {
    #[cfg(feature = "playtest")]
    {
        // Battery/robot runs are driven from arbitrary cwds; the committed
        // default is baked in (mirrors the wasm path) instead of relying on
        // `generate_config_files()` having run in the current directory.
        ron::de::from_bytes(include_bytes!("options/display.ron")).unwrap()
    }
    #[cfg(not(feature = "playtest"))]
    {
        use ron::de::from_str;
        use std::{env::current_dir, fs::read_to_string};

        let config_path = current_dir().unwrap().join("config");

        from_str::<DisplayConfig>(&read_to_string(config_path.join("display.ron")).unwrap())
            .unwrap()
    }
}

#[cfg(target_arch = "wasm32")]
fn get_display_config() -> DisplayConfig {
    DisplayConfig {
        width: 1280.0,
        height: 1024.0,
        fullscreen: false,
    }
}

/// The plugins we need that are "taken for granted" from the engine and basic rendering systems.
/// Using a different `PluginGroupBuilder` is basically a different runtime for the game.
///
/// Generic over the base group: under `--features playtest` the base is
/// `bevy_playtest::PlaytestDefaultPlugins` (a `DefaultPlugins` equivalent
/// that drops `WinitPlugin`/`LogPlugin` under `BEVY_PLAYTEST_*` env vars so
/// the fixture runs headless under takeover); the `.set()` customizations
/// below apply to either.
fn our_default_plugins<G: PluginGroup>(
    group: G,
    display_config: DisplayConfig,
    opts: &options::GameInitCLIOptions,
) -> PluginGroupBuilder {
    let res = group
        .set(WindowPlugin {
            primary_window: Some(Window::from(display_config)),
            ..Default::default()
        })
        .set(ImagePlugin::default_nearest());

    match &opts.assets_dir {
        Some(path_) => res.set(AssetPlugin {
            file_path: path_.to_string_lossy().to_string(),
            ..Default::default()
        }),
        None => res,
    }
}

#[allow(unused_variables, unused_mut)] // The options are only used on some platforms/with some installs
fn our_game_plugins(opts: &GameInitCLIOptions) -> PluginGroupBuilder {
    ThetawaveGamePlugins.build()
}
#[cfg(feature = "playtest")]
fn base_plugin_group() -> bevy_playtest::PlaytestDefaultPlugins {
    bevy_playtest::PlaytestDefaultPlugins
}
#[cfg(not(feature = "playtest"))]
fn base_plugin_group() -> bevy::DefaultPlugins {
    bevy::DefaultPlugins
}

#[cfg(feature = "playtest")]
fn playtest_plugins() -> (bevy_playtest::PlaytestPlugin,) {
    (bevy_playtest::PlaytestPlugin,)
}
#[cfg(not(feature = "playtest"))]
fn playtest_plugins() -> () {}

fn main() {
    // pushes rust errors to the browser console
    #[cfg(target_arch = "wasm32")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    // Playtest/battery runs bake the configs in (see `get_display_config` /
    // `get_input_bindings`); normal native runs still confgen to `config/`.
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "playtest")))]
    generate_config_files();

    let display_config = get_display_config();

    let opts =
        options::GameInitCLIOptions::from_environ_on_supported_platforms_with_default_fallback();
    let mut app = build_app(
        our_default_plugins(base_plugin_group(), display_config, &opts),
        our_game_plugins(&opts),
    );

    // Added last: the runner swap is last-set-wins, so `PlaytestPlugin`
    // must come after every plugin group that might set a runner.
    app.add_plugins(playtest_plugins());

    app.run();
}

/// Make the runnable platform-specific app. `base_plugins` describes "external dependencies"
/// outside the scope of the game itself. These typically come from `bevy::MinimalPlugins` or
/// `bevy::DefaultPlugins`. `game_plugins` comes from from `ThetawaveGamePlugins`.
fn build_app<P1: PluginGroup, P2: PluginGroup>(base_plugins: P1, game_plugins: P2) -> App {
    // Should everything besides adding the plugins be moved into a plugin?
    let mut app = App::new();
    app.add_plugins(base_plugins);
    app.init_state::<AppStates>() // start game in the main menu state
        .init_state::<GameStates>(); // start the game in playing state
    app.add_plugins(game_plugins);
    app.insert_resource(ClearColor(Color::BLACK));
    // Bevy 0.20 port: ambient light is no longer a simple resource insert;
    // tweak the `GlobalAmbientLight` resource installed by `LightPlugin`.
    // Absent in minimal (render-less) test apps, hence the tolerant get.
    if let Some(mut ambient) = app.world_mut().get_resource_mut::<GlobalAmbientLight>() {
        ambient.brightness = AmbientLight::default().brightness;
    }

    app.add_systems(
        OnEnter(AppStates::Game),
        setup_physics.in_set(GameEnterSet::Initialize),
    );
    if cfg!(debug_assertions) && !cfg!(test) {
        app.add_plugins(RapierDebugRenderPlugin::default());
    }
    app
}

// setup rapier
fn setup_physics(
    mut commands: Commands,
    mut rapier_config: Query<&mut RapierConfiguration, With<DefaultRapierContext>>,
) {
    // Bevy 0.20 / bevy_rapier 0.37 port: `timestep_mode` moved out of
    // `RapierConfiguration` into its own resource, and `RapierConfiguration`
    // became a component on the default rapier context entity.
    commands.insert_resource(TimestepMode::Fixed {
        dt: 1.0 / 60.0,
        substeps: 1,
    });
    if let Ok(mut rapier_config) = rapier_config.single_mut() {
        rapier_config.physics_pipeline_active = true;
        rapier_config.gravity = Vec2::ZERO;
    }
}
/// This is the main collection of features and behaviors that define thetawave. 99% of the total
/// behavior of all executables comes from this and Bevy plugins. Ideally 100% of the functionality
/// of all Thetawave executables comes from this and Bevy plugins.
pub struct ThetawaveGamePlugins;
impl PluginGroup for ThetawaveGamePlugins {
    fn build(self) -> PluginGroupBuilder {
        #[allow(unused_mut)] // Allow because we might add more platform-specific features
        let mut res = PluginGroupBuilder::start::<Self>()
            .add(player::PlayerPlugin)
            .add(spawnable::SpawnablePlugin)
            .add(run::RunPlugin)
            .add(loot::LootPlugin)
            .add(game::GamePlugin)
            .add(background::BackgroundPlugin)
            .add(AudioPlugin)
            .add(camera::CameraPlugin)
            .add(arena::ArenaPlugin)
            .add(collision::CollisionPlugin)
            .add(scanner::ScannerPlugin)
            .add(animation::SpriteAnimationPlugin)
            .add(states::StatesPlugin)
            .add(game::counters::plugin::CountingMetricsPlugin)
            .add(health::HealthPlugin)
            .add(weapon::WeaponPlugin)
            .add(
                RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PHYSICS_PIXELS_PER_METER)
                    .in_fixed_schedule(),
            )
            .add(ui::UiPlugin)
            .add(options::OptionsPlugin::default())
            .add(audio::ThetawaveAudioPlugin);
        #[cfg(feature = "storage")]
        {
            res = res.add(thetawave_storage::plugin::DBPlugin);
        }

        res
    }
}

#[cfg(test)]
mod test {
    use crate::animation::SpriteAnimationPlugin;
    use crate::audio::ThetawaveAudioPlugin;
    use crate::background::BackgroundPlugin;
    use crate::{build_app, options, ui, ThetawaveGamePlugins};
    use bevy::app::{App, PluginGroup};
    use bevy::asset::AssetPlugin;
    use bevy::input::InputPlugin;
    use bevy::prelude::{ImagePlugin, NextState, State};
    use bevy::state::app::StatesPlugin;
    use bevy::MinimalPlugins;
    use bevy_kira_audio::AudioPlugin;
    use thetawave_interface::audio::{ChangeBackgroundMusicEvent, PlaySoundEffectEvent};
    use thetawave_interface::game::options::GameOptions;
    use thetawave_interface::states::AppStates;

    #[test]
    fn test_minimal_headless_audioless_game_gets_to_main_menu() {
        // This atleast tests that many resourses/events required by systems have already been
        // inserted, that many assets can be loaded, and that the game can kinda start.
        let mut app = minimal_headless_audioless_app_with_as_many_game_features_as_possible();
        app.update();
        app.world_mut()
            .get_resource_mut::<NextState<AppStates>>()
            .unwrap()
            .set(AppStates::LoadingAssets);
        for _ in 0..10 {
            // just update a few times. Not so important how many.
            app.update();
        }
        app.world_mut()
            .get_resource_mut::<NextState<AppStates>>()
            .unwrap()
            .set(AppStates::MainMenu);
        app.update();
        app.update();
        assert_eq!(
            app.world()
                .get_resource::<State<AppStates>>()
                .unwrap()
                .get(),
            &AppStates::MainMenu
        );
    }

    fn minimal_headless_audioless_app_with_as_many_game_features_as_possible() -> App {
        let base_plugins = MinimalPlugins
            .build()
            .add(AssetPlugin::default())
            .add(ImagePlugin::default())
            .add(InputPlugin)
            .add(StatesPlugin);
        // These features are basically untestable.
        let game_plugins = ThetawaveGamePlugins
            .build()
            // Windowing/display/UI stuff is hard to test and we dont have a screen in CI.
            .disable::<ui::UiPlugin>()
            .disable::<options::OptionsPlugin>()
            // Ideally audio is mostly handled via `thetawave_interface::audio` and events, so that
            // we really only skip testing 1 match statement and external audio deps.
            .disable::<ThetawaveAudioPlugin>()
            .disable::<AudioPlugin>()
            // The background plugin & animation plugins require the render pipeline, which I dont
            // not want in CI.
            .disable::<SpriteAnimationPlugin>()
            .disable::<BackgroundPlugin>();

        let mut app = build_app(base_plugins, game_plugins);
        // bevy_rapier2d 0.37 registers `propagate_parent_transforms`
        // directly without init-ing `StaticTransformOptimizations`
        // (normally done by bevy_transform's TransformPlugin, absent in
        // this minimal render-less app). Supply it manually.
        app.init_resource::<bevy::transform::systems::StaticTransformOptimizations>();
        app.add_message::<ChangeBackgroundMusicEvent>()
            .add_message::<PlaySoundEffectEvent>();

        app.insert_resource(GameOptions::default());
        app
    }
}
