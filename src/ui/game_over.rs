//! System to draw the game over screen.
use crate::{options::PlayingOnArcadeResource, ui::BouncingPromptComponent};
use bevy::{
    asset::AssetServer,
    color::{Alpha, Color},
    ecs::{
        message::MessageWriter,
        system::{Commands, Res},
    },
    text::{Justify, TextColor, TextFont, TextLayout},
    time::{Timer, TimerMode},
    ui::{
        widget::{ImageNode, Text},
        BackgroundColor, FlexDirection, JustifyContent, Node, UiRect, Val,
    },
    utils::default,
};
use std::time::Duration;
use thetawave_assets::UiAssets;
use thetawave_interface::{
    audio::ChangeBackgroundMusicEvent,
    game::historical_metrics::{
        MobKillsByPlayerForCurrentGame, UserStatsByPlayerForCurrentGameCache, DEFAULT_USER_ID,
    },
    states::GameOverCleanup,
};

/// Spawn the styled UI elements for the game over screen. It should tell the player how they did.
pub(super) fn setup_game_over_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_assets: Res<UiAssets>,
    mut change_bg_music_event_writer: MessageWriter<ChangeBackgroundMusicEvent>,
    current_game_shot_counts: Res<UserStatsByPlayerForCurrentGameCache>,
    current_game_enemy_mob_kill_counts: Res<MobKillsByPlayerForCurrentGame>,
    playing_on_arcade: Res<PlayingOnArcadeResource>,
) {
    let maybe_current_game_stats = (**current_game_shot_counts).get(&DEFAULT_USER_ID);
    let (accuracy_rate, total_shots_fired): (f32, usize) = match maybe_current_game_stats {
        None => (100.0, 0),
        Some(current_game_shot_counts) => {
            let accuracy = (current_game_shot_counts.total_shots_hit as f32
                / current_game_shot_counts.total_shots_fired as f32)
                * 100.0;
            (accuracy, current_game_shot_counts.total_shots_fired)
        }
    };

    // fade music out
    change_bg_music_event_writer.write(ChangeBackgroundMusicEvent {
        fade_out: Some(Duration::from_secs(5)),
        ..default()
    });

    commands
        .spawn((Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Default::default()
            }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0))))
        .insert(GameOverCleanup)
        .with_children(|parent| {
            parent
                .spawn((ImageNode {
    image: asset_server.load("texture/game_over_background.png"),
    ..default()
}, Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::FlexEnd,
                        ..Default::default()
                    }, BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 1.0))))
                .with_children(|parent| {
                    let font = ui_assets.lunchds_font.clone();

                    parent
                        .spawn((Node {
                                width: Val::Auto,
                                height: Val::Auto,
                                margin: UiRect {
                                    bottom: Val::Auto,
                                    top: Val::Percent(25.0),
                                    right: Val::Auto,
                                    left: Val::Auto,
                                },
                                padding: UiRect::all(Val::Px(10.0)),

                                justify_content: JustifyContent::Center,
                                ..Default::default()
                            }, BackgroundColor::from(Color::BLACK.with_alpha(0.9))))
                        .with_children(|parent| {
                            parent.spawn((Text::new(format!(
                                        "Projectiles fired: {}\nAccuracy: {:.2}%\n\nEnemies destroyed:\n{}",
                                        total_shots_fired,
                                        accuracy_rate,
                                        super::pprint_mob_kills_from_data(
                                            &current_game_enemy_mob_kill_counts
                                        ),
                                    )), TextFont {
    font: font.into(),
    font_size: 32.0.into(),
    ..Default::default()
}, TextColor(Color::WHITE), TextLayout { justify: Justify::Center, ..Default::default() }, Node {
                                    width: Val::Auto,
                                    height: Val::Auto,
                                    margin: UiRect::all(Val::Auto),
                                    ..Default::default()
                                }));
                        });

                    parent
                        .spawn((ImageNode {
    image: asset_server
                                .load(if **playing_on_arcade {
                                    "texture/restart_game_prompt_arcade.png"
                                } else {
                                    "texture/restart_game_prompt_keyboard.png"
                                }),
    ..default()
}, Node {
                                width: Val::Px(400.0),
                                height: Val::Px(100.0),
                                margin: UiRect {
                                    bottom: Val::Percent(10.0),
                                    top: Val::Auto,
                                    right: Val::Auto,
                                    left: Val::Auto,
                                },
                                justify_content: JustifyContent::Center,
                                ..Default::default()
                            }))
                        .insert(BouncingPromptComponent {
                            flash_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                            is_active: true,
                        });
                });
        });
}
