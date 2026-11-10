#![allow(deprecated)]

use bevy::{
    app::AppExit,
    asset::Handle,
    color::{Color, Srgba},
    ecs::{
        component::Component,
        hierarchy::ChildSpawnerCommands,
        message::{Message, MessageReader, MessageWriter},
        system::ResMut,
    },
    image::{Image, TextureAtlas, TextureAtlasLayout},
    log::info,
    prelude::NextState,
    text::{Font, Justify, TextColor, TextFont, TextLayout},
    ui::{
        widget::{ImageNode, Text},
        AlignItems, BackgroundColor, FlexDirection, JustifyContent, Node, UiRect, Val,
    },
    utils::default,
};
use thetawave_assets::UiAssets;
use thetawave_interface::{player::PlayerInput, states::AppStates};

/// Marker component for clickable menu buttons.
///
/// Bevy 0.20 port: bevy's deprecated `Button` widget became a `pub(crate)`
/// alias for a private struct, so it can no longer be named from downstream
/// crates. This local marker mirrors it; `require(Interaction)` keeps the
/// (deprecated) `ui_focus_system` updating `Interaction` for our buttons.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[require(bevy::ui::Interaction)]
pub(super) struct Button;

/// Event and Component for giving and sending menu buttons actions to move the user from
/// `AppStates::MainMenu` to `AppStates::CharacterSelection`, plus possibly a few digressions and
/// sprinkles.
#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub(super) enum ButtonActionType {
    CharacterSelectReady(u8),
    CharacterSelectJoin,
    CharacterSelectRight(u8),
    CharacterSelectLeft(u8),
    EnterCharacterSelection,
    EnterOptions,
    EnterCompendium,
    QuitGame,
}

#[derive(Component, Clone, PartialEq, Eq, Copy, Debug)]
pub(super) struct ButtonActionComponent {
    pub action: ButtonActionType,
}

impl From<ButtonActionType> for ButtonActionComponent {
    fn from(value: ButtonActionType) -> Self {
        ButtonActionComponent { action: value }
    }
}

#[derive(Message, Clone, PartialEq, Eq, Copy, Debug)]
pub(super) struct ButtonActionEvent {
    pub action: ButtonActionType,
}

impl From<ButtonActionType> for ButtonActionEvent {
    fn from(value: ButtonActionType) -> Self {
        ButtonActionEvent { action: value }
    }
}

impl ButtonActionComponent {
    /// The label that will show on the main menu screen for the button representing this
    /// option/action
    fn text(&self) -> Option<&'static str> {
        match self.action {
            ButtonActionType::EnterCharacterSelection => Some("Start Game"),
            ButtonActionType::EnterOptions => Some("Options"),
            ButtonActionType::EnterCompendium => Some("Compendium"),
            ButtonActionType::QuitGame => Some("Exit Game"),
            ButtonActionType::CharacterSelectLeft(_) => None,
            ButtonActionType::CharacterSelectRight(_) => None,
            ButtonActionType::CharacterSelectJoin => Some("Join"),
            ButtonActionType::CharacterSelectReady(_) => Some("Ready"),
        }
    }

    /// Get input button animations to show on the button
    fn get_input_images(
        &self,
        ui_assets: &UiAssets,
        player_input: Option<PlayerInput>, // optional input method (keyboard/gamepad) of the player that the button belongs to
    ) -> Option<Vec<(Handle<Image>, Handle<TextureAtlasLayout>)>> {
        match self.action {
            ButtonActionType::CharacterSelectJoin => Some(vec![
                (
                    ui_assets.gamepad_button_a_image.clone(),
                    ui_assets.gamepad_button_a_layout.clone(),
                ),
                (
                    ui_assets.keyboard_key_return_image.clone(),
                    ui_assets.keyboard_key_return_layout.clone(),
                ),
            ]),
            ButtonActionType::CharacterSelectReady { .. } => player_input.map(|player_input| {
                vec![
                    (match player_input {
                        PlayerInput::Keyboard => (
                            ui_assets.keyboard_key_return_image.clone(),
                            ui_assets.keyboard_key_return_layout.clone(),
                        ),
                        PlayerInput::Gamepad(_) => (
                            ui_assets.gamepad_button_a_image.clone(),
                            ui_assets.gamepad_button_a_layout.clone(),
                        ),
                    }),
                ]
            }),
            _ => None,
        }
    }

    /// External style applied to buttons
    /// Returns different styles based the based on the button action
    fn get_external_style(&self) -> Node {
        match self.action {
            ButtonActionType::EnterCharacterSelection
            | ButtonActionType::EnterOptions
            | ButtonActionType::EnterCompendium
            | ButtonActionType::QuitGame => Node {
                max_width: Val::Px(500.0),
                width: Val::Percent(25.0),
                min_width: Val::Px(200.0),
                aspect_ratio: Some(160.0 / 34.0),
                margin: UiRect::new(Val::Auto, Val::Auto, Val::Percent(1.0), Val::Percent(1.0)),
                ..default()
            },
            ButtonActionType::CharacterSelectJoin => Node {
                max_width: Val::Px(500.0),
                width: Val::Percent(100.0),
                min_width: Val::Px(200.0),
                aspect_ratio: Some(2.077_922),
                margin: UiRect::new(Val::Auto, Val::Auto, Val::Percent(1.0), Val::Percent(1.0)),
                ..default()
            },
            ButtonActionType::CharacterSelectLeft(_)
            | ButtonActionType::CharacterSelectRight(_) => Node {
                height: Val::Percent(10.0),
                ..default()
            },
            ButtonActionType::CharacterSelectReady { .. } => Node {
                max_width: Val::Px(500.0),
                width: Val::Percent(85.0),
                min_width: Val::Px(300.0),
                margin: UiRect::new(Val::Auto, Val::Auto, Val::Percent(1.0), Val::Percent(1.0)),
                ..default()
            },
        }
    }

    /// Internal style of the ui inside of the button
    /// such as the text and the input symbols
    fn get_internal_style(&self) -> Node {
        match self.action {
            ButtonActionType::EnterCharacterSelection
            | ButtonActionType::EnterOptions
            | ButtonActionType::EnterCompendium
            | ButtonActionType::QuitGame
            | ButtonActionType::CharacterSelectJoin => Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                padding: UiRect::new(Val::ZERO, Val::ZERO, Val::Percent(5.0), Val::ZERO),
                ..default()
            },
            ButtonActionType::CharacterSelectLeft(_)
            | ButtonActionType::CharacterSelectRight(_) => Node {
                padding: UiRect::new(Val::ZERO, Val::ZERO, Val::Percent(5.0), Val::ZERO),
                ..default()
            },
            ButtonActionType::CharacterSelectReady { .. } => Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                padding: UiRect::new(Val::ZERO, Val::ZERO, Val::Percent(5.0), Val::ZERO),
                ..default()
            },
        }
    }

    /// Get image and texture atlas assets based on the button action
    fn asset(&self, ui_assets: &UiAssets) -> (Handle<Image>, Handle<TextureAtlasLayout>) {
        match self.action {
            ButtonActionType::EnterCharacterSelection
            | ButtonActionType::EnterOptions
            | ButtonActionType::EnterCompendium
            | ButtonActionType::QuitGame => (
                ui_assets.thetawave_menu_button_image.clone(),
                ui_assets.thetawave_menu_button_layout.clone(),
            ),
            ButtonActionType::CharacterSelectRight(_) => (
                ui_assets.arrow_right_image.clone(),
                ui_assets.arrow_right_layout.clone(),
            ),
            ButtonActionType::CharacterSelectLeft(_) => (
                ui_assets.arrow_left_image.clone(),
                ui_assets.arrow_left_layout.clone(),
            ),
            ButtonActionType::CharacterSelectJoin => (
                ui_assets.large_menu_button_image.clone(),
                ui_assets.large_menu_button_layout.clone(),
            ),
            ButtonActionType::CharacterSelectReady { .. } => (
                ui_assets.thetawave_menu_button_image.clone(),
                ui_assets.thetawave_menu_button_layout.clone(),
            ),
        }
    }
}

// Handles state changes for buttons. This runs when a user actually
// clicks/whacks enter on a button in the main menu
pub(super) fn button_action_change_state_system(
    mut button_event_reader: MessageReader<ButtonActionEvent>,
    mut next_app_state: ResMut<NextState<AppStates>>,
    mut exit: MessageWriter<AppExit>,
) {
    for event in button_event_reader.read() {
        match event.action {
            ButtonActionType::EnterCharacterSelection => {
                next_app_state.set(AppStates::CharacterSelection);
            }
            ButtonActionType::EnterOptions => info!("Enter options menu."),
            ButtonActionType::EnterCompendium => info!("Enter compendium."),
            ButtonActionType::QuitGame => {
                exit.write(AppExit::Success);
            }
            _ => {}
        }
    }
}

/// Extension trait for spawning customized UI elements for Thetawave
pub(super) trait UiButtonChildBuilderExt {
    /// Spawn a Thetawave-stylized menu button
    fn spawn_button(
        &mut self,
        ui_assets: &UiAssets,
        font: Handle<Font>,
        action: ButtonActionComponent,
        player_input: Option<&PlayerInput>,
    );
}

impl UiButtonChildBuilderExt for ChildSpawnerCommands<'_> {
    fn spawn_button(
        &mut self,
        ui_assets: &UiAssets,
        font: Handle<Font>,
        action: ButtonActionComponent,
        player_input: Option<&PlayerInput>,
    ) {
        // Spawn button bundle entity, with a child entity containing the texture
        self.spawn((
            Button,
            action.get_external_style(),
            BackgroundColor(Color::Srgba(Srgba::NONE)),
        ))
        .insert(action)
        .with_children(|parent| {
            let button_asset = action.asset(ui_assets);

            parent
                .spawn((
                    ImageNode {
                        image: button_asset.0,
                        texture_atlas: Some(TextureAtlas {
                            layout: button_asset.1,
                            ..default()
                        }),
                        ..default()
                    },
                    action.get_internal_style(),
                ))
                .with_children(|parent| {
                    if let Some(text) = action.text() {
                        parent.spawn((
                            Text::new(text),
                            TextFont {
                                font: font.clone().into(),
                                font_size: 30.0.into(),
                                ..Default::default()
                            },
                            TextColor(Color::BLACK),
                            TextLayout {
                                justify: Justify::Center,
                                ..Default::default()
                            },
                        ));
                    }

                    if let Some(inputs) = action.get_input_images(ui_assets, player_input.copied())
                    {
                        // Row for all button inputs
                        parent
                            .spawn(Node {
                                justify_content: JustifyContent::Center,
                                flex_direction: FlexDirection::Row,
                                height: Val::Percent(35.0),
                                margin: UiRect::top(Val::Vh(1.0)),
                                ..default()
                            })
                            .with_children(|parent| {
                                for (image, layout) in inputs {
                                    parent.spawn((
                                        ImageNode {
                                            image,
                                            texture_atlas: Some(TextureAtlas {
                                                layout,
                                                ..default()
                                            }),
                                            ..default()
                                        },
                                        Node {
                                            margin: UiRect {
                                                left: Val::Vw(0.5),
                                                right: Val::Vw(0.5),
                                                ..default()
                                            },
                                            ..default()
                                        },
                                    ));
                                }
                            });
                    }
                });
        });
    }
}
