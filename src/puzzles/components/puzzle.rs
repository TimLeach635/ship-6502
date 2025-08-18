use crate::puzzles::types::*;
use bevy::prelude::Component;

#[derive(Component)]
#[require(Port, AcceptsConnectionStart)]
pub(in crate::puzzles) struct PuzzleInput {
    pub name: String,
}

#[derive(Component)]
#[require(Port, AcceptsConnectionEnd)]
pub(in crate::puzzles) struct PuzzleOutput {
    pub name: String,
}
