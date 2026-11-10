use bevy_ecs::prelude::Message;

pub enum RunOutcomeType {
    Victory,
    Defeat(RunDefeatType),
}

pub enum RunDefeatType {
    PlayersDestroyed,
    DefenseDestroyed,
}

#[derive(Message)]
pub struct RunEndEvent {
    pub outcome: RunOutcomeType,
}

#[derive(Message)]
pub struct CyclePhaseEvent;
