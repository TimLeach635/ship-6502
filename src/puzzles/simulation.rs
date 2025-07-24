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

#[derive(PartialEq)]
enum ResolutionState {
    Unresolved,
    Resolved,
}

#[derive(Component)]
struct Resolvable(ResolutionState);

#[derive(Component)]
pub enum Device {
    Empty,
}

#[derive(Component)]
#[relationship(relationship_target = InputPorts)]
pub struct InputPort(Entity);

#[derive(Component)]
#[relationship_target(relationship = InputPort)]
pub struct InputPorts(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = OutputPorts)]
pub struct OutputPort(Entity);

#[derive(Component)]
#[relationship_target(relationship = OutputPort)]
pub struct OutputPorts(Vec<Entity>);

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

// TODO: It's very possible this should be an exclusive system!
fn resolve(
    q_device_entities: Query<Entity, With<Device>>,
    q_devices: Query<(&Device, Option<&InputPorts>, Option<&OutputPorts>)>,
    mut q_input_ports: Query<&mut Port, (Or<(With<InputPort>, With<IncomingConnections>)>, Without<OutgoingConnection>)>,
    mut q_output_ports: Query<(&mut Port, &OutgoingConnection), (With<OutputPort>, Without<IncomingConnections>)>,
) {
    let mut device_queue: VecDeque<Entity> = VecDeque::new();
    device_queue.extend(q_device_entities);
    let mut visit_counts: EntityHashMap<usize> = EntityHashMap::new();
    visit_counts.extend(q_device_entities.iter().map(|ent| (ent, 0)));  // counts start at 0
    let max_count = device_queue.len();

    while let Some(device_ent) = device_queue.pop_front() {
        let (device, inputs_opt, outputs_opt) = q_devices.get(device_ent)
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
                        .get(&device_ent)
                        .map(|n| *n)
                        .unwrap_or_default();
                    if visit_count < max_count {
                        visit_counts.insert(device_ent, visit_count + 1);
                        device_queue.push_back(device_ent);
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
        // Note - we arbitrarily decide that it is the job of the output ports to pass
        // their value to their connected input ports, rather than the other way round.
        // This is arbitrary, and it could be either way, but we have to be consistent!
        if let Some(outputs) = outputs_opt {
            match device {
                Device::Empty => {
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
        Device::Empty,
        Name::new("Root"),
    )).id();
    let branch_1 = app.world_mut().spawn((
        Device::Empty,
        Name::new("Branch 1"),
    )).id();
    let branch_2 = app.world_mut().spawn((
        Device::Empty,
        Name::new("Branch 2"),
    )).id();
    let leaf_1 = app.world_mut().spawn((
        Device::Empty,
        Name::new("Leaf 1"),
    )).id();
    let leaf_2 = app.world_mut().spawn((
        Device::Empty,
        Name::new("Leaf 2"),
    )).id();
    let leaf_3 = app.world_mut().spawn((
        Device::Empty,
        Name::new("Leaf 3"),
    )).id();

    // Output ports
    let root_o1 = app.world_mut().spawn((
        OutputPort(root),
        Port(None),
        Name::new("Root.o1"),
    )).id();
    let root_o2 = app.world_mut().spawn((
        OutputPort(root),
        Port(None),
        Name::new("Root.o2"),
    )).id();
    let branch1_o1 = app.world_mut().spawn((
        OutputPort(branch_1),
        Port(None),
        Name::new("Branch 1.o1"),
    )).id();
    let branch1_o2 = app.world_mut().spawn((
        OutputPort(branch_1),
        Port(None),
        Name::new("Branch 1.o2"),
    )).id();
    let branch2_o1 = app.world_mut().spawn((
        OutputPort(branch_2),
        Port(None),
        Name::new("Branch 2.o1"),
    )).id();
    let branch2_o2 = app.world_mut().spawn((
        OutputPort(branch_2),
        Port(None),
        Name::new("Branch 2.o2"),
    )).id();
    let leaf1_o1 = app.world_mut().spawn((
        OutputPort(leaf_1),
        Port(None),
        Name::new("Leaf 1.o1"),
    )).id();
    let leaf2_o1 = app.world_mut().spawn((
        OutputPort(leaf_2),
        Port(None),
        Name::new("Leaf 2.o1"),
    )).id();
    let leaf3_o1 = app.world_mut().spawn((
        OutputPort(leaf_3),
        Port(None),
        Name::new("Leaf 3.o1"),
    )).id();

    // Level outputs
    let out_1 = app.world_mut().spawn((
        LevelOutput,
        Port(None),
        Name::new("Level output 1"),
    )).id();
    let out_2 = app.world_mut().spawn((
        LevelOutput,
        Port(None),
        Name::new("Level output 2"),
    )).id();
    let out_3 = app.world_mut().spawn((
        LevelOutput,
        Port(None),
        Name::new("Level output 3"),
    )).id();

    // Input ports
    let branch1_i1 = app.world_mut().spawn((
        InputPort(branch_1),
        Port(None),
        Name::new("Branch 1.i1"),
    )).id();
    let branch2_i1 = app.world_mut().spawn((
        InputPort(branch_2),
        Port(None),
        Name::new("Branch 2.i1"),
    )).id();
    let leaf1_i1 = app.world_mut().spawn((
        InputPort(leaf_1),
        Port(None),
        Name::new("Leaf 1.i1"),
    )).id();
    let leaf2_i1 = app.world_mut().spawn((
        InputPort(leaf_2),
        Port(None),
        Name::new("Leaf 2.i1"),
    )).id();
    let leaf2_i2 = app.world_mut().spawn((
        InputPort(leaf_2),
        Port(None),
        Name::new("Leaf 2.i2"),
    )).id();
    let leaf3_i1 = app.world_mut().spawn((
        InputPort(leaf_3),
        Port(None),
        Name::new("Leaf 3.i1"),
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
