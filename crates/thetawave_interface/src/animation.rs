use bevy_ecs::{entity::Entity, message::Message};

/// This event is used for notifying systems when an animation for an entity has been completed
/// Can be used for despawning entities after animations finish
#[derive(Message)]
pub struct AnimationCompletedEvent(pub Entity);
