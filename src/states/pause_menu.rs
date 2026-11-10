//! Systems to (set the game states to) open and close the pause menu.
use crate::audio;
use bevy::prelude::{AssetServer, NextState, Query, Res, ResMut, With};
use bevy_kira_audio::prelude::{AudioChannel, AudioControl};
use bevy_rapier2d::plugin::context::DefaultRapierContext;
use bevy_rapier2d::prelude::RapierConfiguration;
use leafwing_input_manager::prelude::ActionState;
use thetawave_interface::input::{MainMenuExplorer, MenuAction};
use thetawave_interface::states::GameStates;

pub(super) fn open_pause_menu_system(
    menu_input_query: Query<&ActionState<MenuAction>, With<MainMenuExplorer>>,
    mut next_game_state: ResMut<NextState<GameStates>>,
    mut rapier_config: Query<&mut RapierConfiguration, With<DefaultRapierContext>>,
    asset_server: Res<AssetServer>,
    audio_channel: Res<AudioChannel<audio::MenuAudioChannel>>,
) {
    let Ok(action_state) = menu_input_query.single() else {
        return;
    };

    // swiitch to pause menu state if input read
    if action_state.just_released(&MenuAction::PauseGame) {
        next_game_state.set(GameStates::Paused);

        // play sound effect
        audio_channel.play(asset_server.load("sounds/menu_input_success.wav"));

        // suspend the physics engine (Bevy 0.20 port: `RapierConfiguration`
        // is now a component on the default rapier context entity)
        rapier_config.single_mut().unwrap().physics_pipeline_active = false;
        // query_pipeline_active removed upstream in 0.37 (always-on via SimulationMode)
    }
}

// close pause menu if input given
pub(super) fn close_pause_menu_system(
    menu_input_query: Query<&ActionState<MenuAction>, With<MainMenuExplorer>>,
    mut next_game_state: ResMut<NextState<GameStates>>,
    mut rapier_config: Query<&mut RapierConfiguration, With<DefaultRapierContext>>,
    asset_server: Res<AssetServer>,
    audio_channel: Res<AudioChannel<audio::MenuAudioChannel>>,
) {
    // read menu input action
    let Ok(action_state) = menu_input_query.single() else {
        return;
    };

    // pop the pause state if input read
    if action_state.just_released(&MenuAction::ExitPauseMenu) {
        next_game_state.set(GameStates::Playing);

        // play sound effect
        audio_channel.play(asset_server.load("sounds/menu_input_success.wav"));

        // resume the physics engine
        rapier_config.single_mut().unwrap().physics_pipeline_active = true;
    }
}
