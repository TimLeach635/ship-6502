use std::collections::VecDeque;
use bevy::ecs::entity::EntityHashMap;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StepSimulation>()
            .add_systems(
                PreUpdate,
                resolve.run_if(on_event::<StepSimulation>),
            );
    }
}

#[derive(Event)]
pub struct StepSimulation;

pub enum DeviceKind {
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
pub struct Device {
    pub kind: DeviceKind,
}

#[derive(Component)]
#[relationship(relationship_target = InputPorts)]
pub struct InputPortOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = InputPortOf)]
pub struct InputPorts(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = OutputPorts)]
pub struct OutputPortOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = OutputPortOf)]
pub struct OutputPorts(Vec<Entity>);

#[derive(Component)]
struct Name(String);

// TODO: Pretty certain that these relationships are the wrong way round!
//  "OutgoingConnection" on an Entity should mean that this Entity IS an
//  outgoing connection, not that it HAS one!
#[derive(Component)]
#[relationship(relationship_target = IncomingConnections)]
pub struct OutgoingConnection(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = OutgoingConnection)]
pub struct IncomingConnections(Vec<Entity>);

/// Represents an output port of the entire level.
#[derive(Component)]
struct LevelOutput;

#[derive(Component)]
// TODO: Too specific? Should this be ValueHolder?
pub struct Port(pub Option<u32>);

fn resolve(
    q_device_entities: Query<Entity, With<Device>>,
    q_devices: Query<(Option<&InputPorts>, Option<&OutputPorts>), With<Device>>,
    mut q_input_ports: Query<&mut Port, (Or<(With<InputPortOf>, With<IncomingConnections>)>, Without<OutgoingConnection>)>,
    mut q_output_ports: Query<(&mut Port, &OutgoingConnection), (With<OutputPortOf>, Without<IncomingConnections>)>,
) {
    let mut device_queue: VecDeque<Entity> = VecDeque::new();
    device_queue.extend(q_device_entities);
    let mut visit_counts: EntityHashMap<usize> = EntityHashMap::new();
    visit_counts.extend(q_device_entities.iter().map(|ent| (ent, 0)));  // counts start at 0
    let max_count = device_queue.len();

    while let Some(device) = device_queue.pop_front() {
        let (inputs_opt, outputs_opt) = q_devices.get(device)
            .expect("Device should always be in query");

        // We can resolve this device if all the inputs are known.
        // (For future - maybe we can even if some of them aren't!)
        // When we do that, we should set the values of the outputs accordingly,
        // to allow other devices to also resolve.
        if let Some(inputs) = inputs_opt {  // If no inputs then it always resolves
            for input_entity in inputs.iter() {
                let input_port = q_input_ports.get(input_entity)
                    .expect("Should not have an input port without a Port component");
                if input_port.0.is_none() {
                    // At least one of the input ports of this device has no value,
                    // so (for now!) we assume we cannot resolve.
                    // If we've already pushed this device to the queue a certain
                    // number of times, then we're almost certainly in a cycle.
                    // TODO: This is actually a really inefficient way of traversing the
                    //  device tree - replace this with something less naive.
                    let visit_count = visit_counts
                        .get(&device)
                        .map(|n| *n)
                        .unwrap_or_default();
                    if visit_count < max_count {
                        visit_counts.insert(device, visit_count + 1);
                        device_queue.push_back(device);
                        continue;
                    } else {
                        // TODO: Handle gracefully. In the game this definitely should not panic
                        //  and instead will just be handled. This is not a crash situation!
                        panic!("Encountered a cycle");
                    }
                }
            }
        }

        // If we reach this point, all the input ports have known values
        // TODO: Actually perform device-specific processing of the inputs
        // For now, just set all the output ports of this device to 0.
        // Note - we arbitrarily decide that it is the job of the output ports to pass
        // their value to their connected input ports, rather than the other way round.
        // This is arbitrary, and it could be either way, but we have to be consistent!
        if let Some(outputs) = outputs_opt {
            for output_entity in outputs.iter() {
                let (mut output_port, outgoing_connection) = q_output_ports.get_mut(output_entity)
                    .expect("Should not have an output port without a Port component");
                let mut connected_input_port = q_input_ports.get_mut(outgoing_connection.get())
                    .expect("Should not have an input port without a Port component");
                let value: u32 = 0;  // This will be what changes based on the actual device
                output_port.0 = Some(value);
                connected_input_port.0 = Some(value);
            }
        }
    }
}

fn reset_resolutions(q_resolvable: Query<&mut Resolvable>) {
    for mut resolvable in q_resolvable {
        resolvable.0 = ResolutionState::Unresolved;
    }
}

#[test]
fn can_resolve_port_values_in_a_circuit_without_panicking() {
    let mut app = App::new();

    // Devices
    let root = app.world_mut().spawn((
        Device {
            kind: DeviceKind::Empty,
        },
        Name("Root".to_owned()),
    )).id();
    let branch_1 = app.world_mut().spawn((
        Device {
            kind: DeviceKind::Empty,
        },
        Name("Branch 1".to_owned()),
    )).id();
    let branch_2 = app.world_mut().spawn((
        Device {
            kind: DeviceKind::Empty,
        },
        Name("Branch 2".to_owned()),
    )).id();
    let leaf_1 = app.world_mut().spawn((
        Device {
            kind: DeviceKind::Empty,
        },
        Name("Leaf 1".to_owned()),
    )).id();
    let leaf_2 = app.world_mut().spawn((
        Device {
            kind: DeviceKind::Empty,
        },
        Name("Leaf 2".to_owned()),
    )).id();
    let leaf_3 = app.world_mut().spawn((
        Device {
            kind: DeviceKind::Empty,
        },
        Name("Leaf 3".to_owned()),
    )).id();

    // Output ports
    let root_o1 = app.world_mut().spawn((
        OutputPortOf(root),
        Port(None),
        Name("Root.o1".to_owned()),
    )).id();
    let root_o2 = app.world_mut().spawn((
        OutputPortOf(root),
        Port(None),
        Name("Root.o2".to_owned()),
    )).id();
    let branch1_o1 = app.world_mut().spawn((
        OutputPortOf(branch_1),
        Port(None),
        Name("Branch 1.o1".to_owned()),
    )).id();
    let branch1_o2 = app.world_mut().spawn((
        OutputPortOf(branch_1),
        Port(None),
        Name("Branch 1.o2".to_owned()),
    )).id();
    let branch2_o1 = app.world_mut().spawn((
        OutputPortOf(branch_2),
        Port(None),
        Name("Branch 2.o1".to_owned()),
    )).id();
    let branch2_o2 = app.world_mut().spawn((
        OutputPortOf(branch_2),
        Port(None),
        Name("Branch 2.o2".to_owned()),
    )).id();
    let leaf1_o1 = app.world_mut().spawn((
        OutputPortOf(leaf_1),
        Port(None),
        Name("Leaf 1.o1".to_owned()),
    )).id();
    let leaf2_o1 = app.world_mut().spawn((
        OutputPortOf(leaf_2),
        Port(None),
        Name("Leaf 2.o1".to_owned()),
    )).id();
    let leaf3_o1 = app.world_mut().spawn((
        OutputPortOf(leaf_3),
        Port(None),
        Name("Leaf 3.o1".to_owned()),
    )).id();

    // Level outputs
    let out_1 = app.world_mut().spawn((
        LevelOutput,
        Port(None),
        Name("Level output 1".to_owned()),
    )).id();
    let out_2 = app.world_mut().spawn((
        LevelOutput,
        Port(None),
        Name("Level output 2".to_owned()),
    )).id();
    let out_3 = app.world_mut().spawn((
        LevelOutput,
        Port(None),
        Name("Level output 3".to_owned()),
    )).id();

    // Input ports
    let branch1_i1 = app.world_mut().spawn((
        InputPortOf(branch_1),
        Port(None),
        Name("Branch 1.i1".to_owned()),
    )).id();
    let branch2_i1 = app.world_mut().spawn((
        InputPortOf(branch_2),
        Port(None),
        Name("Branch 2.i1".to_owned()),
    )).id();
    let leaf1_i1 = app.world_mut().spawn((
        InputPortOf(leaf_1),
        Port(None),
        Name("Leaf 1.i1".to_owned()),
    )).id();
    let leaf2_i1 = app.world_mut().spawn((
        InputPortOf(leaf_2),
        Port(None),
        Name("Leaf 2.i1".to_owned()),
    )).id();
    let leaf2_i2 = app.world_mut().spawn((
        InputPortOf(leaf_2),
        Port(None),
        Name("Leaf 2.i2".to_owned()),
    )).id();
    let leaf3_i1 = app.world_mut().spawn((
        InputPortOf(leaf_3),
        Port(None),
        Name("Leaf 3.i1".to_owned()),
    )).id();

    // Connections
    app.world_mut().entity_mut(root_o1).insert(OutgoingConnection(branch1_i1));
    app.world_mut().entity_mut(root_o2).insert(OutgoingConnection(branch2_i1));
    app.world_mut().entity_mut(branch1_o1).insert(OutgoingConnection(leaf1_i1));
    app.world_mut().entity_mut(branch1_o2).insert(OutgoingConnection(leaf2_i1));
    app.world_mut().entity_mut(branch2_o1).insert(OutgoingConnection(leaf2_i2));
    app.world_mut().entity_mut(branch2_o2).insert(OutgoingConnection(leaf3_i1));
    app.world_mut().entity_mut(leaf1_o1).insert(OutgoingConnection(out_1));
    app.world_mut().entity_mut(leaf2_o1).insert(OutgoingConnection(out_2));
    app.world_mut().entity_mut(leaf3_o1).insert(OutgoingConnection(out_3));

    // Systems
    app.add_systems(Update, resolve);

    // Perform update
    app.update();

    // Confirm that all ports now have values
    assert!(app.world().get::<Port>(root_o1).unwrap().0.is_some());
    assert!(app.world().get::<Port>(root_o2).unwrap().0.is_some());
    assert!(app.world().get::<Port>(branch1_o1).unwrap().0.is_some());
    assert!(app.world().get::<Port>(branch1_o2).unwrap().0.is_some());
    assert!(app.world().get::<Port>(branch2_o1).unwrap().0.is_some());
    assert!(app.world().get::<Port>(branch2_o2).unwrap().0.is_some());
    assert!(app.world().get::<Port>(leaf1_o1).unwrap().0.is_some());
    assert!(app.world().get::<Port>(leaf2_o1).unwrap().0.is_some());
    assert!(app.world().get::<Port>(leaf3_o1).unwrap().0.is_some());
    assert!(app.world().get::<Port>(out_1).unwrap().0.is_some());
    assert!(app.world().get::<Port>(out_2).unwrap().0.is_some());
    assert!(app.world().get::<Port>(out_3).unwrap().0.is_some());
    assert!(app.world().get::<Port>(branch1_i1).unwrap().0.is_some());
    assert!(app.world().get::<Port>(branch2_i1).unwrap().0.is_some());
    assert!(app.world().get::<Port>(leaf1_i1).unwrap().0.is_some());
    assert!(app.world().get::<Port>(leaf2_i1).unwrap().0.is_some());
    assert!(app.world().get::<Port>(leaf2_i2).unwrap().0.is_some());
    assert!(app.world().get::<Port>(leaf3_i1).unwrap().0.is_some());
}
