use bevy::prelude::*;
use crate::puzzles::types::*;

#[derive(Component)]
pub(in crate::puzzles) enum Device {
    Constant {
        value: u32,
        output_port: Entity,
    },
    Counter {
        value: u32,
        output_port: Entity,
    },
    Repeater {
        input_port: Entity,
        output_port: Entity,
    },
    Adder {
        input_port_1: Entity,
        input_port_2: Entity,
        output_port: Entity,
    },
}

#[derive(Component)]
#[relationship(relationship_target = InputPorts)]
#[require(AcceptsConnectionEnd)]
pub(in crate::puzzles) struct InputPort(Entity);

#[derive(Component)]
#[relationship_target(relationship = InputPort)]
pub(in crate::puzzles) struct InputPorts(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = OutputPorts)]
#[require(AcceptsConnectionStart)]
pub(in crate::puzzles) struct OutputPort(Entity);

#[derive(Component)]
#[relationship_target(relationship = OutputPort)]
pub(in crate::puzzles) struct OutputPorts(Vec<Entity>);

#[derive(Component, Default)]
// TODO: Too specific? Should this be ValueHolder?
pub struct Port(pub Option<u32>);
