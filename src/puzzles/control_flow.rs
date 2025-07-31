//! Control flow (pause, reset, step, etc.) for the simulation.
//!
//! Adapted from [`cellular-automata-demo`](https://github.com/alice-i-cecile/cellular-automata-demo)
//! by [@alice-i-cecile](https://github.com/alice-i-cecile). Many thanks for sending this to me!

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

pub struct ControlFlowPlugin;

impl Plugin for ControlFlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ResetSimulation>()
            .add_event::<StepSimulation>();
    }
}

#[derive(Event)]
pub struct ResetSimulation;

#[derive(Event)]
pub struct StepSimulation;

#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Default)]
pub enum SimState {
    #[default]
    NotRunning,
    Running,
    Paused,
}
