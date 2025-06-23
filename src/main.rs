use std::cmp::PartialEq;
use std::collections::VecDeque;
use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::relationship::{Relationship, RelationshipSourceCollection};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_systems(Startup, (
            test_startup,
            resolve.after(test_startup)
        ))
        .add_systems(Update, reset_resolutions)
        .run();
}

enum DeviceKind {
    Empty,
}

#[derive(PartialEq)]
enum ResolutionState {
    Unresolved,
    Resolved,
}

#[derive(Component)]
struct Resolvable(ResolutionState);

#[derive(Component)]
struct Device {
    kind: DeviceKind,
}

#[derive(Component)]
#[relationship(relationship_target = InputPorts)]
struct InputPortOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = InputPortOf)]
struct InputPorts(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = OutputPorts)]
struct OutputPortOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = OutputPortOf)]
struct OutputPorts(Vec<Entity>);

#[derive(Component)]
struct Name(String);

#[derive(Component)]
#[relationship(relationship_target = IncomingConnections)]
struct OutgoingConnection(Entity);

#[derive(Component)]
#[relationship_target(relationship = OutgoingConnection)]
struct IncomingConnections(Vec<Entity>);

fn test_startup(mut commands: Commands) {
    // Devices
    let root = commands.spawn((
        Device {
            kind: DeviceKind::Empty,
        },
        Name("Root".to_string()),
    )).id();
    let branch_1 = commands.spawn((
        Device {
            kind: DeviceKind::Empty,
        },
        Name("Branch 1".to_string()),
    )).id();
    let branch_2 = commands.spawn((
        Device {
            kind: DeviceKind::Empty,
        },
        Name("Branch 2".to_string()),
    )).id();
    let leaf_1 = commands.spawn((
        Device {
            kind: DeviceKind::Empty,
        },
        Name("Leaf 1".to_string()),
    )).id();
    let leaf_2 = commands.spawn((
        Device {
            kind: DeviceKind::Empty,
        },
        Name("Leaf 2".to_string()),
    )).id();
    let leaf_3 = commands.spawn((
        Device {
            kind: DeviceKind::Empty,
        },
        Name("Leaf 3".to_string()),
    )).id();
    
    // Output ports
    let root_o1 = commands.spawn((
        OutputPortOf(root),
        Name("Root.o1".to_string()),
    )).id();
    let root_o2 = commands.spawn((
        OutputPortOf(root),
        Name("Root.o2".to_string()),
    )).id();
    let branch1_o1 = commands.spawn((
        OutputPortOf(branch_1),
        Name("Branch 1.o1".to_string()),
    )).id();
    let branch1_o2 = commands.spawn((
        OutputPortOf(branch_1),
        Name("Branch 1.o2".to_string()),
    )).id();
    let branch2_o1 = commands.spawn((
        OutputPortOf(branch_2),
        Name("Branch 2.o1".to_string()),
    )).id();
    let branch2_o2 = commands.spawn((
        OutputPortOf(branch_2),
        Name("Branch 2.o2".to_string()),
    )).id();
    
    // Input ports
    let branch1_i1 = commands.spawn((
        InputPortOf(branch_1),
        Name("Branch 1.i1".to_string()),
    )).id();
    let branch2_i1 = commands.spawn((
        InputPortOf(branch_2),
        Name("Branch 2.i1".to_string()),
    )).id();
    let leaf1_i1 = commands.spawn((
        InputPortOf(leaf_1),
        Name("Leaf 1.i1".to_string()),
    )).id();
    let leaf2_i1 = commands.spawn((
        InputPortOf(leaf_2),
        Name("Leaf 2.i1".to_string()),
    )).id();
    let leaf2_i2 = commands.spawn((
        InputPortOf(leaf_2),
        Name("Leaf 2.i2".to_string()),
    )).id();
    let leaf3_i1 = commands.spawn((
        InputPortOf(leaf_3),
        Name("Leaf 3.i1".to_string()),
    )).id();
    
    // Connections
    commands.entity(root_o1).insert(OutgoingConnection(branch1_i1));
    commands.entity(root_o2).insert(OutgoingConnection(branch2_i1));
    commands.entity(branch1_o1).insert(OutgoingConnection(leaf1_i1));
    commands.entity(branch1_o2).insert(OutgoingConnection(leaf2_i1));
    commands.entity(branch2_o1).insert(OutgoingConnection(leaf2_i2));
    commands.entity(branch2_o2).insert(OutgoingConnection(leaf3_i1));
}

fn resolve(
    q_root_devices: Query<Entity, (With<Device>, Without<InputPorts>)>,
    q_devices: Query<(&Name, Option<&OutputPorts>), With<Device>>,
    q_output_ports: Query<&OutgoingConnection, With<OutputPortOf>>,
    q_input_ports: Query<&InputPortOf, With<IncomingConnections>>,
) {
    // Start at the devices that don't have any input ports, as it is these devices
    //  whose behaviour is known right away
    let mut device_queue: VecDeque<Entity> = VecDeque::new();
    let mut visited = EntityHashSet::new();
    device_queue.extend(q_root_devices);
    visited.extend(q_root_devices);
    
    // After we have traversed the graph starting at the input-less devices, if there are
    //  any devices remaining that we haven't visited, then we have a problem.
    // Note to self: perhaps it's fine to have devices that can't resolve? Like, if they
    //  have a device that's just disconnected from all the others, it could just sit there
    //  in an unresolved state. Maybe this is okay? Maybe there are only a subset of devices
    //  whose resolution is required each step. Maybe none of them are, and the puzzles are
    //  designed to have special ports that input and output.
    // Shit, maybe that's how this should work. We don't start at input-less devices, we
    //  have some ports that aren't attached to any device, and they feed data into the system,
    //  following through until they hit an input port (which is an output of the whole system)
    //  that isn't attached to any device.
    // Of course, I could actually implement that by having special devices that own the ports
    //  that are like "pseudo-devices" that don't visually appear in the game.
    // Fuck, it's hard implementing this when I don't even know what the puzzles will look like.
    // Maybe I need to, you know, actually design the game before making it?
    while let Some(device) = device_queue.pop_front() {
        if let Ok((
            Name(name),
            outputs_option
        )) = q_devices.get(device) {
            println!("Visiting device \"{}\"", name);
            
            // Get all output ports of this device
            if let Some(outputs) = outputs_option {
                for output_entity in outputs.iter() {
                    if let Ok(connection) = q_output_ports.get(output_entity) {
                        let input_entity = connection.get();
                        if let Ok(input) = q_input_ports.get(input_entity) {
                            let connected_device = input.get();
                            if visited.add(connected_device) {
                                device_queue.push_back(connected_device);
                            }
                        } else {
                            println!("Input port not part of a device");
                        }
                    } else {
                        println!("Output port not connected to anything");
                    }
                }
            } else {
                println!("  Device \"{}\" has no output ports", name);
            }
        }
    }
}

fn reset_resolutions(q_resolvable: Query<&mut Resolvable>) {
    for mut resolvable in q_resolvable {
        resolvable.0 = ResolutionState::Unresolved;
    }
}
