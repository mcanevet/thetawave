//! Systems to draw and update UI elements related to the player's level progression and objectives
//! (e.x. health). The user should know that certain behaviors bring them closer to defeat and know
//! how far away they are from losing.
use crate::run::CurrentRunProgressResource;
use bevy::{
    asset::Handle,
    color::{Alpha, Color, Srgba},
    ecs::{
        component::Component,
        entity::Entity,
        hierarchy::ChildSpawnerCommands,
        query::With,
        system::{Commands, Query, Res},
    },
    text::{Font, Justify, TextColor, TextFont, TextLayout},
    ui::{
        widget::Text, AlignItems, BackgroundColor, FlexDirection, JustifyContent, Node, UiRect, Val,
    },
    utils::default,
};

use super::parent::LevelUiChildBuilderExt;

const NODE_WIDTH: Val = Val::Percent(50.0);
const TEXT_COLOR: Srgba = Srgba::WHITE;
const FONT_SIZE: f32 = 48.0;
const LEVEL_DATA_PADDING: UiRect =
    UiRect::new(Val::Vw(1.0), Val::Vw(1.0), Val::Vh(2.0), Val::Vh(2.0));
const DEFENSE_COLOR: Srgba = Srgba::BLUE;
const DEFENSE_COLOR_EMPTY_ALPHA: f32 = 0.05;
const DEFENSE_COLOR_FILLED_ALPHA: f32 = 0.75;
const DEFENSE_WIDTH: Val = Val::Percent(80.0);
const DEFENSE_HEIGHT: Val = Val::Percent(60.0);

/// Used for querying UI for displaying name
#[derive(Component)]
pub(super) struct LevelNameUi;

/// Used for querying UI for displaying level information
#[derive(Component)]
pub(super) struct LevelDataUi;

impl LevelUiChildBuilderExt for ChildSpawnerCommands<'_> {
    fn spawn_level_ui(&mut self, font: Handle<Font>) {
        self.spawn(Node {
            width: NODE_WIDTH,
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|left| {
            left.spawn((
                Text::new(""),
                TextFont {
                    font: font.clone().into(),
                    font_size: FONT_SIZE.into(),
                    ..Default::default()
                },
                TextColor(Color::Srgba(TEXT_COLOR)),
                TextLayout {
                    justify: Justify::Center,
                    ..Default::default()
                },
                Node::default(),
            ))
            .insert(LevelNameUi);
        });

        self.spawn(Node {
            width: NODE_WIDTH,
            height: Val::Percent(100.0),
            ..default()
        })
        .with_children(|right| {
            right
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: LEVEL_DATA_PADDING,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .insert(LevelDataUi);
        });
    }
}

/// Updates the all of the level ui at the bottom of the window
pub(super) fn update_level_ui_system(
    mut commands: Commands,
    level_data_ui_query: Query<Entity, With<LevelDataUi>>,
    mut level_name_ui_query: Query<&mut Text, With<LevelNameUi>>,
    run_resource: Res<CurrentRunProgressResource>,
) {
    if let Some(current_level) = &run_resource.current_level {
        if let Ok(mut text) = level_name_ui_query.single_mut() {
            text.0 = current_level.get_name().to_string();
        }

        if let Ok(entity) = level_data_ui_query.single() {
            commands.entity(entity).despawn_children();

            if let Some(objective) = &current_level.objective {
                match objective {
                    thetawave_interface::objective::Objective::Defense(defense_data) => {
                        commands.entity(entity).with_children(|level_data_ui| {
                            level_data_ui
                                .spawn((
                                    Node {
                                        width: DEFENSE_WIDTH,
                                        height: DEFENSE_HEIGHT,
                                        flex_direction: FlexDirection::Row,
                                        ..default()
                                    },
                                    BackgroundColor(
                                        DEFENSE_COLOR.with_alpha(DEFENSE_COLOR_EMPTY_ALPHA).into(),
                                    ),
                                ))
                                .with_children(|defense_ui| {
                                    defense_ui.spawn((
                                        Node {
                                            width: Val::Percent(
                                                100.0 * defense_data.get_percentage(),
                                            ),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        BackgroundColor(
                                            DEFENSE_COLOR
                                                .with_alpha(DEFENSE_COLOR_FILLED_ALPHA)
                                                .into(),
                                        ),
                                    ));
                                });
                        });
                    }
                }
            }
        }
    }
}
