use crate::run::level_phase::LevelPhaseType;
use crate::run::tutorial::TutorialLesson;
use crate::{run::CurrentRunProgressResource, spawnable::BossComponent};
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
        widget::Text, AlignItems, BackgroundColor, FlexDirection, FlexWrap, JustifyContent, Node,
        UiRect, Val,
    },
    utils::default,
};
use thetawave_assets::UiAssets;
use thetawave_interface::health::HealthComponent;

use super::parent::PhaseUiChildBuilderExt;

const NODE_WIDTH: Val = Val::Percent(50.0);
const NORMAL_TEXT_COLOR: Srgba = Srgba::WHITE;
const TUTORIAL_COMPLETED_TEXT_COLOR: Srgba = Srgba::GREEN;
const TUTORIAL_FONT_SIZE: f32 = 24.0;
const FONT_SIZE: f32 = 48.0;
const PHASE_DATA_PADDING: UiRect =
    UiRect::new(Val::Vw(1.0), Val::Vw(1.0), Val::Vh(2.0), Val::Vh(2.0));
const BOSS_HEALTH_WIDTH: Val = Val::Percent(80.0);
const BOSS_HEALTH_HEIGHT: Val = Val::Percent(60.0);
const BOSS_HEALTH_COLOR: Srgba = Srgba::RED;
const BOSS_HEALTH_EMPTY_ALPHA: f32 = 0.05;
const BOSS_HEALTH_FILLED_ALPHA: f32 = 0.75;
const TUTORIAL_TEXT_SECTION_HEIGHT: Val = Val::Px(30.0);

/// Used for querying UI for displaying name
#[derive(Component)]
pub(super) struct PhaseNameUi;

/// Used for querying UI for displaying phase information
#[derive(Component)]
pub(super) struct PhaseDataUi;

impl PhaseUiChildBuilderExt for ChildSpawnerCommands<'_> {
    // Phase name UI
    fn spawn_phase_ui(&mut self, font: Handle<Font>) {
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
                    font: font.into(),
                    font_size: FONT_SIZE.into(),
                    ..Default::default()
                },
                TextColor(Color::Srgba(NORMAL_TEXT_COLOR)),
                TextLayout {
                    justify: Justify::Center,
                    ..Default::default()
                },
                Node::default(),
            ))
            .insert(PhaseNameUi);
        });

        // Phase data ui
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
                    padding: PHASE_DATA_PADDING,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .insert(PhaseDataUi);
        });
    }
}

pub(super) fn update_phase_ui_system(
    ui_assets: Res<UiAssets>,
    mut commands: Commands,
    mut phase_name_ui_query: Query<&mut Text, With<PhaseNameUi>>,
    phase_data_ui_query: Query<Entity, With<PhaseDataUi>>,
    run_resource: Res<CurrentRunProgressResource>,
    boss_mobs_query: Query<&HealthComponent, With<BossComponent>>,
) {
    if let Some(current_level) = &run_resource.current_level {
        if let Some(current_phase) = &current_level.current_phase {
            if let Ok(mut text) = phase_name_ui_query.single_mut() {
                text.0 = current_phase.phase_type.get_name().to_string()
            }

            if let Ok(entity) = phase_data_ui_query.single() {
                commands.entity(entity).despawn_children();

                match &current_phase.phase_type {
                    LevelPhaseType::FormationSpawn { phase_timer, .. } => {
                        let font = ui_assets.lunchds_font.clone();

                        commands.entity(entity).with_children(|phase_data_ui| {
                            phase_data_ui.spawn((
                                Text::new(format!("{:.0}", phase_timer.remaining_secs())),
                                TextFont {
                                    font: font.into(),
                                    font_size: FONT_SIZE.into(),
                                    ..Default::default()
                                },
                                TextColor(Color::Srgba(NORMAL_TEXT_COLOR)),
                                TextLayout {
                                    justify: Justify::Center,
                                    ..Default::default()
                                },
                                Node::default(),
                            ));
                        });
                    }
                    LevelPhaseType::Break { phase_timer, .. } => {
                        let font = ui_assets.lunchds_font.clone();

                        commands.entity(entity).with_children(|phase_data_ui| {
                            phase_data_ui.spawn((
                                Text::new(format!("{:.0}", phase_timer.remaining_secs())),
                                TextFont {
                                    font: font.into(),
                                    font_size: FONT_SIZE.into(),
                                    ..Default::default()
                                },
                                TextColor(Color::Srgba(NORMAL_TEXT_COLOR)),
                                TextLayout {
                                    justify: Justify::Center,
                                    ..Default::default()
                                },
                                Node::default(),
                            ));
                        });
                    }
                    LevelPhaseType::Boss { .. } => {
                        if let Ok(health) = boss_mobs_query.single() {
                            commands.entity(entity).with_children(|phase_data_ui| {
                                phase_data_ui
                                    .spawn((
                                        Node {
                                            width: BOSS_HEALTH_WIDTH,
                                            height: BOSS_HEALTH_HEIGHT,
                                            flex_direction: FlexDirection::Row,
                                            ..default()
                                        },
                                        BackgroundColor(
                                            BOSS_HEALTH_COLOR
                                                .with_alpha(BOSS_HEALTH_EMPTY_ALPHA)
                                                .into(),
                                        ),
                                    ))
                                    .with_children(|boss_health_ui| {
                                        boss_health_ui.spawn((
                                            Node {
                                                width: Val::Percent(
                                                    100.0 * health.get_health_percentage(),
                                                ),
                                                height: Val::Percent(100.0),
                                                ..default()
                                            },
                                            BackgroundColor(
                                                BOSS_HEALTH_COLOR
                                                    .with_alpha(BOSS_HEALTH_FILLED_ALPHA)
                                                    .into(),
                                            ),
                                        ));
                                    });
                            });
                        }
                    }
                    LevelPhaseType::Tutorial {
                        tutorial_lesson, ..
                    } => {
                        let font = ui_assets.lunchds_font.clone();

                        commands.entity(entity).with_children(|phase_data_ui| {
                            phase_data_ui
                                .spawn(Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    flex_wrap: FlexWrap::Wrap,
                                    ..default()
                                })
                                .with_children(|phase_data_list_ui| match tutorial_lesson {
                                    TutorialLesson::Movement { .. } => {
                                        for (progress_str, completed) in
                                            tutorial_lesson.get_movement_timer_strs().iter()
                                        {
                                            phase_data_list_ui.spawn((
                                                Text::new(progress_str),
                                                TextFont {
                                                    font: font.clone().into(),
                                                    font_size: TUTORIAL_FONT_SIZE.into(),
                                                    ..Default::default()
                                                },
                                                TextColor(if *completed {
                                                    Color::Srgba(TUTORIAL_COMPLETED_TEXT_COLOR)
                                                } else {
                                                    Color::Srgba(NORMAL_TEXT_COLOR)
                                                }),
                                                TextLayout {
                                                    justify: Justify::Center,
                                                    ..Default::default()
                                                },
                                                Node {
                                                    height: TUTORIAL_TEXT_SECTION_HEIGHT,
                                                    ..default()
                                                },
                                            ));
                                        }
                                    }
                                    TutorialLesson::AbilitySlotOne { .. } => {
                                        for (progress_str, completed) in
                                            tutorial_lesson.get_attack_strs().iter()
                                        {
                                            phase_data_list_ui.spawn((
                                                Text::new(progress_str),
                                                TextFont {
                                                    font: font.clone().into(),
                                                    font_size: TUTORIAL_FONT_SIZE.into(),
                                                    ..Default::default()
                                                },
                                                TextColor(if *completed {
                                                    Color::Srgba(TUTORIAL_COMPLETED_TEXT_COLOR)
                                                } else {
                                                    Color::Srgba(NORMAL_TEXT_COLOR)
                                                }),
                                                TextLayout {
                                                    justify: Justify::Center,
                                                    ..Default::default()
                                                },
                                                Node {
                                                    height: TUTORIAL_TEXT_SECTION_HEIGHT,
                                                    ..default()
                                                },
                                            ));
                                        }
                                    }
                                    TutorialLesson::AbilitySlotTwo { .. } => {
                                        for (progress_str, completed) in
                                            tutorial_lesson.get_ability_strs().iter()
                                        {
                                            phase_data_list_ui.spawn((
                                                Text::new(progress_str),
                                                TextFont {
                                                    font: font.clone().into(),
                                                    font_size: TUTORIAL_FONT_SIZE.into(),
                                                    ..Default::default()
                                                },
                                                TextColor(if *completed {
                                                    Color::Srgba(TUTORIAL_COMPLETED_TEXT_COLOR)
                                                } else {
                                                    Color::Srgba(NORMAL_TEXT_COLOR)
                                                }),
                                                TextLayout {
                                                    justify: Justify::Center,
                                                    ..Default::default()
                                                },
                                                Node {
                                                    height: TUTORIAL_TEXT_SECTION_HEIGHT,
                                                    ..default()
                                                },
                                            ));
                                        }
                                    }
                                });
                        });
                    }
                }
            }
        }
    }
}
