use bevy::prelude::{Component, Entity};

#[derive(Component, Default)]
pub(in crate::puzzles) struct AcceptsConnectionStart;

#[derive(Component, Default)]
pub(in crate::puzzles) struct AcceptsConnectionEnd;

#[derive(Component)]
#[relationship(relationship_target = ConnectionEnds)]
pub(in crate::puzzles) struct ConnectionStart(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = ConnectionStart)]
pub(in crate::puzzles) struct ConnectionEnds(Vec<Entity>);
