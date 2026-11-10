use bevy::{
    asset::Handle,
    color::{Color, Srgba},
    ecs::system::{Commands, Res},
    image::Image,
    text::Font,
    ui::{AlignItems, BackgroundColor, FlexDirection, JustifyContent, Node, Val},
    utils::default,
};
use thetawave_assets::UiAssets;
use thetawave_interface::{
    abilities::AbilitySlotIDComponent,
    character::Character,
    player::{PlayerIDComponent, PlayersResource},
    states::GameCleanup,
};

use crate::player::CharactersResource;

use super::border_gradient::BorderGradientType;

const TOP_ROW_HEIGHT: Val = Val::Percent(13.0);
const TOP_CORNER_WIDTH: Val = Val::Percent(10.0);
const TOP_CENTER_WIDTH: Val = Val::Percent(80.0);
const BOTTOM_ROW_HEIGHT: Val = Val::Percent(13.0);
const BOTTOM_CORNER_WIDTH: Val = Val::Percent(10.0);
const BOTTOM_CENTER_WIDTH: Val = Val::Percent(80.0);
const OUTSIDE_BORDER_BG_COLOR: BackgroundColor =
    BackgroundColor(Color::Srgba(Srgba::new(0.0, 0.0, 0.0, 0.0)));
const MIDDLE_ROW_HEIGHT: Val = Val::Percent(74.0);
const MIDDLE_SIDE_WIDTH: Val = Val::Percent(10.0);
const MIDDLE_CENTER_WIDTH: Val = Val::Percent(80.0);

pub(super) trait BorderGradientCommandsExt {
    fn spawn_border_gradient(&mut self, ui_assets: &UiAssets, bg_type: BorderGradientType);
}

pub(super) trait PhaseUiChildBuilderExt {
    fn spawn_phase_ui(&mut self, font: Handle<Font>);
}

pub(super) trait LevelUiChildBuilderExt {
    fn spawn_level_ui(&mut self, font: Handle<Font>);
}

pub(super) trait GameCenterUiChildBuilderExt {
    fn spawn_game_center_ui(&mut self, font: Handle<Font>);
}

pub(super) trait PlayerUiChildBuilderExt {
    fn spawn_player_ui(
        &mut self,
        characters_res: &CharactersResource,
        id: PlayerIDComponent,
        players_res: &PlayersResource,
        ui_assets: &UiAssets,
    );
    fn spawn_inner_player_ui(&mut self, id: PlayerIDComponent);
    fn spawn_outer_player_ui(
        &mut self,
        character: &Character,
        id: PlayerIDComponent,
        ui_assets: &UiAssets,
    );
    fn spawn_player_ability_slot_ui(
        &mut self,
        character: &Character,
        player_id: PlayerIDComponent,
        ability_slot_id: AbilitySlotIDComponent,
        is_flipped: bool,
        ui_assets: &UiAssets,
    );
    fn spawn_player_armor_counter_ui(&mut self);
    fn spawn_player_ability_icon_ui(
        &mut self,
        player_id: PlayerIDComponent,
        ability_slot_id: AbilitySlotIDComponent,
        icon: Handle<Image>,
    );
}

/// initializes the game ui hierarchy
pub(super) fn setup_game_ui_system(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    players_resource: Res<PlayersResource>,
    characters_resource: Res<CharactersResource>,
) {
    let font: Handle<Font> = ui_assets.lunchds_font.clone();

    commands.spawn_border_gradient(&ui_assets, BorderGradientType::Defense);
    commands.spawn_border_gradient(&ui_assets, BorderGradientType::Warning);

    // Spawn the top level Node for all game ui and all of its child entities
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .insert(GameCleanup)
        .with_children(|game| {
            // Parent node for top row containing the phase ui
            game.spawn(Node {
                width: Val::Percent(100.0),
                height: TOP_ROW_HEIGHT,
                flex_direction: FlexDirection::Row,
                ..default()
            })
            .with_children(|top_row| {
                // Top left corner
                top_row.spawn((
                    Node {
                        width: TOP_CORNER_WIDTH,
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    OUTSIDE_BORDER_BG_COLOR,
                ));

                // Top middle
                top_row
                    .spawn((
                        Node {
                            width: TOP_CENTER_WIDTH,
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        OUTSIDE_BORDER_BG_COLOR,
                    ))
                    .with_children(|top_middle| {
                        // spawn the phase ui
                        top_middle.spawn_phase_ui(font.clone());
                    });

                // Top right corner
                top_row.spawn((
                    Node {
                        width: TOP_CORNER_WIDTH,
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    OUTSIDE_BORDER_BG_COLOR,
                ));
            });

            // Middle Row
            game.spawn(Node {
                width: Val::Percent(100.0),
                height: MIDDLE_ROW_HEIGHT,
                flex_direction: FlexDirection::Row,
                ..default()
            })
            .with_children(|middle_row| {
                // Left column on the left side of window excluding the corners
                middle_row
                    .spawn((
                        Node {
                            width: MIDDLE_SIDE_WIDTH,
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        OUTSIDE_BORDER_BG_COLOR,
                    ))
                    .with_children(|middle_left| {
                        // Player 1 Ui on the left
                        middle_left.spawn_player_ui(
                            &characters_resource,
                            PlayerIDComponent::One,
                            &players_resource,
                            &ui_assets,
                        );
                    });

                // Middle column over the top of the arena
                middle_row
                    .spawn(Node {
                        width: MIDDLE_CENTER_WIDTH,
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|middle_center| {
                        // spawn the ui for displaying messages in the center of the game
                        middle_center.spawn_game_center_ui(font.clone());
                    });

                // Left column on the left side of window excluding the corners
                middle_row
                    .spawn((
                        Node {
                            width: MIDDLE_SIDE_WIDTH,
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        OUTSIDE_BORDER_BG_COLOR,
                    ))
                    .with_children(|middle_right| {
                        // Player 1 Ui onm the left
                        middle_right.spawn_player_ui(
                            &characters_resource,
                            PlayerIDComponent::Two,
                            &players_resource,
                            &ui_assets,
                        );
                    });
            });

            game.spawn(Node {
                width: Val::Percent(100.0),
                height: BOTTOM_ROW_HEIGHT,
                flex_direction: FlexDirection::Row,
                ..default()
            })
            .with_children(|bottom_row| {
                // Top left corner
                bottom_row.spawn((
                    Node {
                        width: BOTTOM_CORNER_WIDTH,
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    OUTSIDE_BORDER_BG_COLOR,
                ));

                // Bottom middle
                bottom_row
                    .spawn((
                        Node {
                            width: BOTTOM_CENTER_WIDTH,
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        OUTSIDE_BORDER_BG_COLOR,
                    ))
                    .with_children(|bottom_middle| {
                        // spawn the level ui
                        bottom_middle.spawn_level_ui(font);
                    });

                // Bottom right corner
                bottom_row.spawn((
                    Node {
                        width: BOTTOM_CORNER_WIDTH,
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    OUTSIDE_BORDER_BG_COLOR,
                ));
            });
        });
}
