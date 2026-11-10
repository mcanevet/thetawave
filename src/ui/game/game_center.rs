//! Systems that draw things that go in the center of the screen.
use crate::run::CurrentRunProgressResource;
use bevy::{
    asset::Handle,
    color::{Alpha, Color, Srgba},
    ecs::hierarchy::ChildSpawnerCommands,
    ecs::{
        component::Component,
        message::MessageReader,
        query::With,
        system::{Query, Res},
    },
    text::{Font, Justify, TextColor, TextFont, TextLayout},
    time::{Time, Timer, TimerMode},
    ui::{widget::Text, BackgroundColor, Node},
};
use std::time::Duration;
use thetawave_interface::run::CyclePhaseEvent;

use super::parent::GameCenterUiChildBuilderExt;

const BASE_TEXT_ALPHA: f32 = 1.0;
const BASE_BACKGROUND_ALPHA: f32 = 0.4;
const FONT_SIZE: f32 = 90.0;
const TEXT_COLOR: Srgba = Srgba::WHITE;
const BACKGROUND_COLOR: Srgba = Srgba::BLACK;
const DEFAULT_FADE_TIME: f32 = 5.0;

#[derive(Component)]
pub(super) struct CenterTextUi;

#[derive(Component)]
pub(super) struct FadeOutUiComponent {
    pub timer: Timer,
}

impl Default for FadeOutUiComponent {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(DEFAULT_FADE_TIME, TimerMode::Once);
        timer.set_elapsed(Duration::from_secs_f32(DEFAULT_FADE_TIME));

        Self { timer }
    }
}

impl GameCenterUiChildBuilderExt for ChildSpawnerCommands<'_> {
    fn spawn_game_center_ui(&mut self, font: Handle<Font>) {
        self.spawn((
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
            BackgroundColor(Color::Srgba(BACKGROUND_COLOR.with_alpha(0.0))),
        ))
        .insert(CenterTextUi)
        .insert(FadeOutUiComponent::default());
    }
}

/// Updates the alpha color of the background color for the text ui
pub(super) fn update_center_text_ui_system(
    mut cycle_phase_event_reader: MessageReader<CyclePhaseEvent>,
    run_resource: Res<CurrentRunProgressResource>,
    mut center_text_query: Query<
        (&mut Text, &mut BackgroundColor, &mut FadeOutUiComponent),
        With<CenterTextUi>,
    >,
) {
    // if phase has been cycled update the text
    if cycle_phase_event_reader.read().next().is_some() {
        if let Some(level) = &run_resource.current_level {
            if let Some(phase) = &level.current_phase {
                if let Ok((mut text, mut bg_color, mut fade_out)) = center_text_query.single_mut() {
                    if let Some(intro_text) = phase.intro_text.clone() {
                        text.0 = intro_text;
                        *bg_color = BACKGROUND_COLOR.with_alpha(BASE_BACKGROUND_ALPHA).into();
                        fade_out.timer.reset();
                    }
                }
            }
        }
    }
}

/// Gradually fade out text entities with a `FadeOutUIComponent`. This should be run every frame.
pub(super) fn text_fade_out_system(
    mut background_color_query: Query<(
        &mut Text,
        &mut TextColor,
        &mut BackgroundColor,
        &mut FadeOutUiComponent,
    )>,
    time: Res<Time>,
) {
    for (_text, mut text_color, mut bg_color, mut fade_out) in background_color_query.iter_mut() {
        fade_out.timer.tick(time.delta());

        *bg_color = BACKGROUND_COLOR
            .with_alpha(BASE_BACKGROUND_ALPHA * fade_out.timer.fraction_remaining())
            .into();

        *text_color = TextColor(Color::Srgba(
            TEXT_COLOR.with_alpha(BASE_TEXT_ALPHA * fade_out.timer.fraction_remaining()),
        ));
    }
}
