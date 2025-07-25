use std::collections::VecDeque;
use bevy::ecs::entity::EntityHashMap;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use crate::puzzles::devices::Device;
use crate::ui::buttons::SpawnButtonCommandExt;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StepSimulation>();
        app.add_systems(Startup, setup);
        app.add_systems(
            PreUpdate,
            resolve.run_if(on_event::<StepSimulation>),
        );
        app.add_systems(
            Update,
            // TODO: If I make this only run on the event, it doesn't incorporate the values
            //  from the resolution. I suspect that this is because `resolve` isn't an exclusive
            //  system, but I could be wrong. For example, surely the PreUpdate stuff will all
            //  complete and the commands are run before Update happens?
            update_port_value_display/*.run_if(on_event::<StepSimulation>)*/,
        );
    }
}

#[derive(Event)]
pub struct StepSimulation;

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

#[derive(Component)]
#[relationship(relationship_target = ConnectionEnds)]
pub struct ConnectionStart(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = ConnectionStart)]
pub struct ConnectionEnds(Vec<Entity>);

/// Represents an output port of the entire level.
#[derive(Component)]
struct LevelOutput;

#[derive(Component)]
// TODO: Too specific? Should this be ValueHolder?
pub struct Port(pub Option<u32>);

fn setup(mut commands: Commands) {
    // Buttons for simulation control
    let button_parent = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::End,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )).id();

    let step_button = commands.spawn_button(120.0, "Step", on_click_step);
    commands.entity(button_parent).add_child(step_button);
}

fn on_click_step(
    _: Trigger<Pointer<Click>>,
    mut ev_step: EventWriter<StepSimulation>,
) {
    ev_step.write(StepSimulation);
}

// TODO: It's very possible this should be an exclusive system!
fn resolve(
    q_device_entities: Query<Entity, With<Device>>,
    mut q_devices: Query<(&mut Device, Option<&InputPorts>, Option<&OutputPorts>)>,
    mut q_input_ports: Query<&mut Port, (Or<(With<InputPort>, With<ConnectionEnds>)>, (Without<OutputPort>, Without<ConnectionStart>))>,
    mut q_output_ports: Query<(&mut Port, Option<&ConnectionStart>), (With<OutputPort>, Without<ConnectionEnds>)>,
) {
    let mut device_queue: VecDeque<Entity> = VecDeque::new();
    device_queue.extend(q_device_entities);
    let mut visit_counts: EntityHashMap<usize> = EntityHashMap::new();
    visit_counts.extend(q_device_entities.iter().map(|ent| (ent, 0)));  // counts start at 0
    let max_count = device_queue.len();

    while let Some(device_ent) = device_queue.pop_front() {
        let (mut device, inputs_opt, outputs_opt) = q_devices.get_mut(device_ent)
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
                        panic!("Unresolvable");
                    }
                }
            }
        }

        // If we reach this point, all the input ports have known values
        // Note - we arbitrarily decide that it is the job of the output ports to pass
        // their value to their connected input ports, rather than the other way round.
        // This is arbitrary, and it could be either way, but we have to be consistent!
        if outputs_opt.is_some() {
            match *device {
                Device::Constant { value, output_port } => {
                    let (mut output_port, outgoing_conn_opt) = q_output_ports.get_mut(output_port)
                        .expect("Should not have an output port without a Port component");
                    output_port.0 = Some(value);

                    if let Some(outgoing_conn) = outgoing_conn_opt {
                        let mut connected_input_port = q_input_ports.get_mut(outgoing_conn.get())
                            .expect("Should not have an input port without a Port component");
                        connected_input_port.0 = Some(value);
                    }
                },
                Device::Counter { value, output_port: out_ent } => {
                    let (mut output_port, outgoing_conn_opt) = q_output_ports.get_mut(out_ent)
                        .expect("Should not have an output port without a Port component");
                    output_port.0 = Some(value);

                    if let Some(outgoing_conn) = outgoing_conn_opt {
                        let mut connected_input_port = q_input_ports.get_mut(outgoing_conn.get())
                            .expect("Should not have an input port without a Port component");
                        connected_input_port.0 = Some(value);
                    }

                    // Then update the counter
                    // TODO: I don't think this should happen here, but I want to get some more
                    //  devices written before sorting it
                    *device = Device::Counter { value: value + 1, output_port: out_ent };
                },
                Device::Repeater { input_port, output_port } => {
                    let input_port_value = q_input_ports.get(input_port)
                        .expect("Directly-specified input port should be an input port")
                        .0
                        .expect("At this point, all the input ports to this device should have known values");
                    
                    
                    let (mut output_port, outgoing_conn_opt) = q_output_ports.get_mut(output_port)
                        .expect("Should not have an output port without a Port component");
                    output_port.0 = Some(input_port_value);

                    if let Some(outgoing_conn) = outgoing_conn_opt {
                        let mut connected_input_port = q_input_ports.get_mut(outgoing_conn.get())
                            .expect("Should not have an input port without a Port component");
                        connected_input_port.0 = Some(input_port_value);
                    }
                },
                Device::Adder { input_port_1, input_port_2, output_port } => {
                    let input_port_1_value = q_input_ports.get(input_port_1)
                        .expect("Directly-specified input port should be an input port")
                        .0
                        .expect("At this point, all the input ports to this device should have known values");
                    let input_port_2_value = q_input_ports.get(input_port_2)
                        .expect("Directly-specified input port should be an input port")
                        .0
                        .expect("At this point, all the input ports to this device should have known values");
                    let sum = input_port_1_value + input_port_2_value;


                    let (mut output_port, outgoing_conn_opt) = q_output_ports.get_mut(output_port)
                        .expect("Should not have an output port without a Port component");
                    output_port.0 = Some(sum);

                    if let Some(outgoing_conn) = outgoing_conn_opt {
                        let mut connected_input_port = q_input_ports.get_mut(outgoing_conn.get())
                            .expect("Should not have an input port without a Port component");
                        connected_input_port.0 = Some(sum);
                    }
                },
            }
        }
    }
}

fn update_port_value_display(
    q_ports: Query<(&Port, &Children)>,
    mut q_text: Query<&mut Text2d>,
) {
    for (port, children) in q_ports {
        for child_ent in children {
            if let Ok(mut text) = q_text.get_mut(*child_ent) {
                text.0 = if let Some(value) = port.0 {
                    value.to_string()
                } else {
                    "<none>".to_owned()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::puzzles::devices::{DeviceKind, SpawnDeviceCommandExt};
    use super::*;

    #[test]
    fn constant_device_outputs_its_value() {
        let mut app = App::new();

        let mut meshes: Assets<Mesh> = default();
        let mut materials: Assets<ColorMaterial> = default();

        let constant_device_entities = app.world_mut().commands()
            .spawn_device(DeviceKind::Constant { value: 10 }, &mut meshes, &mut materials);
        let output_port_ent = constant_device_entities.output_ports.first().unwrap();

        // Systems
        app.add_systems(Update, resolve);

        // Perform update
        app.update();

        // Confirm that output port has been given the right value
        let result = app.world().get::<Port>(*output_port_ent).unwrap().0;
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 10);
    }

    #[test]
    fn complicated_network_can_resolve_all_devices() {
        let mut app = App::new();

        let mut meshes: Assets<Mesh> = default();
        let mut materials: Assets<ColorMaterial> = default();

        // Devices
        let const_1_ents = app.world_mut().commands()
            .spawn_device(DeviceKind::Constant { value: 1 }, &mut meshes, &mut materials);
        let const_2_ents = app.world_mut().commands()
            .spawn_device(DeviceKind::Constant { value: 2 }, &mut meshes, &mut materials);
        let repeater_1_ents = app.world_mut().commands()
            .spawn_device(DeviceKind::Repeater, &mut meshes, &mut materials);
        let repeater_2_ents = app.world_mut().commands()
            .spawn_device(DeviceKind::Repeater, &mut meshes, &mut materials);
        let adder_ents = app.world_mut().commands()
            .spawn_device(DeviceKind::Adder, &mut meshes, &mut materials);

        // Connections
        //   const_1 -> repeater_1
        app.world_mut().commands().entity(repeater_1_ents.input_ports[0])
            .add_one_related::<ConnectionStart>(const_1_ents.output_ports[0]);
        //   const_2 -> repeater_2
        app.world_mut().commands().entity(repeater_2_ents.input_ports[0])
            .add_one_related::<ConnectionStart>(const_2_ents.output_ports[0]);
        //   repeater_1 -> adder
        app.world_mut().commands().entity(adder_ents.input_ports[0])
            .add_one_related::<ConnectionStart>(repeater_1_ents.output_ports[0]);
        //   repeater_2 -> adder
        app.world_mut().commands().entity(adder_ents.input_ports[1])
            .add_one_related::<ConnectionStart>(repeater_2_ents.output_ports[0]);

        // Systems
        app.add_systems(Update, resolve);

        // Perform update
        app.update();

        // Confirm that adder's output port is the sum of the two constants
        let result = app.world().get::<Port>(adder_ents.output_ports[0]).unwrap().0;
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 3);
    }
}
