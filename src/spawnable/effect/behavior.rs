use crate::animation::AnimationComponent;
use crate::GameUpdateSet;
use bevy::app::{App, Plugin, Update};
use bevy::asset::Assets;
use bevy::color::Alpha;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageReader;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, Query, Res};
use bevy::image::TextureAtlasLayout;
use bevy::sprite::{Sprite, Text2d};
use bevy::state::condition::in_state;
use bevy::text::TextColor;
use bevy::time::{Stopwatch, Time, Timer, TimerMode};
use serde::Deserialize;
use thetawave_interface::animation::AnimationCompletedEvent;
use thetawave_interface::states;

use super::EffectComponent;

/// `EffectBehaviorPlugin` manages the behaviors of in-game effects.
///
/// This plugin is responsible for updating and managing the behaviors of effects during the game.
/// It adds systems to the app which handle the behavior of effects based on their type and state,
/// such as despawning effects after their animation completes, or fading out text and sprite effects.
pub struct EffectBehaviorPlugin;

impl Plugin for EffectBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                despawn_after_animation_effect_behavior_system
                    .in_set(GameUpdateSet::ExecuteBehavior),
                fade_out_text_effect_behavior_system.in_set(GameUpdateSet::ExecuteBehavior),
                fade_out_sprite_effect_behavior_system.in_set(GameUpdateSet::ExecuteBehavior),
                fade_out_despawn_after_animation_effect_behavior_system
                    .in_set(GameUpdateSet::ExecuteBehavior),
            )
                .run_if(in_state(states::AppStates::Game))
                .run_if(in_state(states::GameStates::Playing)),
        );
    }
}

/// Enumerates the types of behaviors that can be performed by effects,
/// uses data types for tracking and updating the behavior.
#[derive(Deserialize, Clone)]
pub enum EffectBehavior {
    DespawnAfterAnimation,
    FadeOut(Timer),
    FadeOutAndDespawnAfterAnimation(Stopwatch),
}

/// Enumerates the types of behaviors that can be performed by effects using
/// primitive data types for initializing `EffectBehavior` counterparts.
#[derive(Deserialize, Clone)]
pub enum EffectBehaviorData {
    DespawnAfterAnimation,
    // Takes a f32 time in seconds to take fading out before despawning
    FadeOut(f32),
    FadeOutAndDespawnAfterAnimation,
}

impl From<EffectBehaviorData> for EffectBehavior {
    fn from(value: EffectBehaviorData) -> Self {
        match value {
            EffectBehaviorData::DespawnAfterAnimation => EffectBehavior::DespawnAfterAnimation,
            EffectBehaviorData::FadeOut(seconds) => {
                EffectBehavior::FadeOut(Timer::from_seconds(seconds, TimerMode::Once))
            }
            EffectBehaviorData::FadeOutAndDespawnAfterAnimation => {
                EffectBehavior::FadeOutAndDespawnAfterAnimation(Stopwatch::new())
            }
        }
    }
}

/// Checks if each effect entity has a `DespawnAfterAnimation` behavior.
/// Recursively despawns the effect entities with this behavior after
/// its last animation frame is complete.
fn despawn_after_animation_effect_behavior_system(
    mut commands: Commands,
    effect_query: Query<(Entity, &EffectComponent, &Sprite, &AnimationComponent)>,
    texture_atlas_layouts: Res<Assets<TextureAtlasLayout>>,
) {
    // Check if entity has  an `DespawnAfterAnimation` behavior
    for (entity, effect_component, sprite, animation) in effect_query.iter() {
        if effect_component
            .behaviors
            .iter()
            .any(|behavior| matches!(behavior, EffectBehavior::DespawnAfterAnimation))
        {
            // Despawn effect entity after animation is complete
            // Bevy 0.20 port: the atlas lives on the `Sprite`, not as a component.
            if let Some(texture_atlas) = sprite.texture_atlas.as_ref() {
                if let Some(texture_atlas_layout) =
                    texture_atlas_layouts.get(texture_atlas.layout.id())
                {
                    if texture_atlas.index == texture_atlas_layout.textures.len() - 1
                        && animation.timer.just_finished()
                    {
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}

/// Checks if each effect entity with a `Text2d` component has a `FadeOut` behavior.
/// Recursively despawns the effect entities with this behavior after
/// the timer is complete, while also fading out linearly based on the percent of time left in the timer.
fn fade_out_text_effect_behavior_system(
    mut commands: Commands,
    mut effect_query: Query<(Entity, &mut EffectComponent, &mut Text2d, &mut TextColor)>,
    time: Res<Time>,
) {
    for (entity, mut effect_component, _text, mut text_color) in effect_query.iter_mut() {
        if let Some(timer) = effect_component.behaviors.iter_mut().find_map(|behavior| {
            if let EffectBehavior::FadeOut(timer) = behavior {
                Some(timer)
            } else {
                None
            }
        }) {
            timer.tick(time.delta());

            // if the timer just completed, recursively despawn the effect entity, otherwise change the alpha
            if timer.just_finished() {
                commands.entity(entity).despawn();
            } else {
                // Fade the text color alpha out linearly
                text_color.0.set_alpha(timer.fraction_remaining());
            }
        }
    }
}

/// Checks if each effect entity with a `TextureAtlasSprite` component has a `FadeOutMs` behavior.
/// Recursively despawns the effect entities with this behavior after
/// the timer is complete, while also fading out linearly based on the percent of time left in the timer.
fn fade_out_sprite_effect_behavior_system(
    mut commands: Commands,
    mut effect_query: Query<(Entity, &mut EffectComponent, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut effect_component, mut sprite) in effect_query.iter_mut() {
        if let Some(timer) = effect_component.behaviors.iter_mut().find_map(|behavior| {
            if let EffectBehavior::FadeOut(timer) = behavior {
                Some(timer)
            } else {
                None
            }
        }) {
            timer.tick(time.delta());

            // if the timer just completed, recursively despawn the effect entity, otherwise change the alpha
            if timer.just_finished() {
                commands.entity(entity).despawn();
            } else {
                sprite.color.set_alpha(timer.fraction_remaining());
            }
        }
    }
}

/// Checks if each effect entity has a `FadeOutAndDespawnAfterAnimation` behavior.
/// Recursively despawns the effect entities with this behavior after
/// the animation is complete, while also fading out along an exponential decay curve.
fn fade_out_despawn_after_animation_effect_behavior_system(
    mut commands: Commands,
    mut animation_completed_event_reader: MessageReader<AnimationCompletedEvent>,
    mut effect_query: Query<(
        Entity,
        &mut EffectComponent,
        &mut Sprite,
        &AnimationComponent,
    )>,
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlasLayout>>,
) {
    let animation_completed_events: Vec<&AnimationCompletedEvent> =
        animation_completed_event_reader.read().collect();

    for (entity, mut effect_component, mut sprite, animation) in effect_query.iter_mut() {
        // Bevy 0.20 port: the atlas layout handle lives on the `Sprite`, not as a component.
        let Some(texture_atlas_layout_handle) =
            sprite.texture_atlas.as_ref().map(|a| a.layout.clone())
        else {
            continue;
        };
        if let (Some(texture_atlas_layout), Some(stopwatch)) = (
            texture_atlases.get(texture_atlas_layout_handle.id()),
            effect_component.behaviors.iter_mut().find_map(|behavior| {
                if let EffectBehavior::FadeOutAndDespawnAfterAnimation(stopwatch) = behavior {
                    Some(stopwatch)
                } else {
                    None
                }
            }),
        ) {
            // Despawn if the animation is completed, otherwise continue fading out
            if animation_completed_events.iter().any(|e| e.0 == entity) {
                commands.entity(entity).despawn();
            } else {
                stopwatch.tick(time.delta());

                // Get the total animation time duration of a frame * the number of frames
                let total_animation_time = animation.timer.duration().as_secs_f32()
                    * texture_atlas_layout.textures.len() as f32;

                // Get an alpha value along an exponential decay curve
                let elapsed_time = stopwatch.elapsed().as_secs_f32();
                let decay_constant = -(total_animation_time.recip()) * 0.001_f32.ln();
                let alpha = (1.0_f32 * (-decay_constant * elapsed_time).exp()).clamp(0.0, 1.0);

                sprite.color.set_alpha(alpha);
            }
        }
    }
}
