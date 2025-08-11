use crate::puzzles::item_placement::{AcceptsConnectionStart, AcceptsConnectionEnd};
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
            Update,
            (
                resolve
                    .run_if(on_event::<StepSimulation>),
                advance_devices
                    .run_if(on_event::<StepSimulation>)
                    .after(resolve),
                update_port_value_display
                    .run_if(on_event::<StepSimulation>)
                    .after(resolve),
                advance_step_number
                    .run_if(on_event::<StepSimulation>),
                update_step_number_display
                    .run_if(on_event::<StepSimulation>)
                    .after(advance_step_number),
            )
        );
    }
}

#[derive(Resource)]
pub struct SimulationStep(pub(crate) usize);

#[derive(Event)]
pub struct StepSimulation;

#[derive(Component)]
#[relationship(relationship_target = InputPorts)]
#[require(AcceptsConnectionEnd)]
pub struct InputPort(Entity);

#[derive(Component)]
#[relationship_target(relationship = InputPort)]
pub struct InputPorts(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = OutputPorts)]
#[require(AcceptsConnectionStart)]
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

#[derive(Component, Default)]
// TODO: Too specific? Should this be ValueHolder?
pub struct Port(pub Option<u32>);

#[derive(Component)]
#[require(Text)]
struct SimulationStepLabel;

fn setup(mut commands: Commands) {
    commands.insert_resource(SimulationStep(0));

    // Text for which step it is
    commands.spawn((
        SimulationStepLabel,
        Text::new("Step 0"),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            right: Val::Px(12.0),
            ..default()
        },
    ));

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

fn resolve(
    world: &mut World,
) {
    let mut q_device_entities = world.query_filtered::<Entity, With<Device>>();

    let mut device_queue: VecDeque<Entity> = VecDeque::new();
    device_queue.extend(q_device_entities.iter(world));
    let mut visit_counts: EntityHashMap<usize> = EntityHashMap::new();
    visit_counts.extend(q_device_entities.iter(world).map(|ent| (ent, 0)));  // counts start at 0
    let max_count = device_queue.len();

    'entities: while let Some(device_ent) = device_queue.pop_front() {
        let device = world.get(device_ent).unwrap();
        let inputs_opt = world.get::<InputPorts>(device_ent);
        let outputs_opt = world.get::<OutputPorts>(device_ent);

        // We can resolve this device if all the inputs are known.
        // (For future - maybe we can even if some of them aren't!)
        // When we do that, we should set the values of the outputs accordingly,
        // to allow other devices to also resolve.
        if let Some(inputs) = inputs_opt {  // If no inputs then it always resolves
            for input_entity in inputs.iter() {
                let input_port = world.get::<Port>(input_entity)
                    // TODO: Either add as a required component, or consolidate the components
                    .expect("Should not have an input port without a Port component");
                
                // If no connections end on this component, then we cannot resolve
                if world.get::<ConnectionEnds>(input_entity).is_none() {
                    todo!("Gracefully handle when an input is not connected");
                }
                
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
                        continue 'entities;
                    } else {
                        todo!("Gracefully handle a probable cycle");
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
                Device::Constant { value, output_port: out_ent } => {
                    set_output_value_and_update_connection_if_exists(world, out_ent, value);
                },
                Device::Counter { value, output_port: out_ent } => {
                    set_output_value_and_update_connection_if_exists(world, out_ent, value);
                },
                Device::Repeater { input_port, output_port: out_ent } => {
                    let input_port_value = world.get::<Port>(input_port)
                        .expect("Directly-specified input port should be an input port")
                        .0
                        .expect("At this point, all the input ports to this device should have known values");

                    set_output_value_and_update_connection_if_exists(world, out_ent, input_port_value);
                },
                Device::Adder { input_port_1, input_port_2, output_port: out_ent } => {
                    let input_port_1_value = world.get::<Port>(input_port_1)
                        .expect("Directly-specified input port should be an input port")
                        .0
                        .expect("At this point, all the input ports to this device should have known values");
                    let input_port_2_value = world.get::<Port>(input_port_2)
                        .expect("Directly-specified input port should be an input port")
                        .0
                        .expect("At this point, all the input ports to this device should have known values");
                    let sum = input_port_1_value + input_port_2_value;

                    set_output_value_and_update_connection_if_exists(world, out_ent, sum);
                },
            }
        }
    }
}

/// What a verbosely-named function!
///
/// When passed an output port entity, this function:
/// 1. Sets its value to the provided one
/// 2. Checks to see if there is a connection beginning at that port
/// 3. If there is, also sets the value of the connected port to the same value
// TODO: Should this function return a Result?
fn set_output_value_and_update_connection_if_exists(
    world: &mut World,
    output_port_entity: Entity,
    value: u32,
) {
    let mut output_port = world.get_mut::<Port>(output_port_entity)
        .expect("Should not have an output port without a Port component");
    output_port.0 = Some(value);

    if let Some(connection_start) = world.get::<ConnectionStart>(output_port_entity) {
        let mut connected_input_port = world.get_mut::<Port>(connection_start.get())
            .expect("Should not have an input port without a Port component");
        connected_input_port.0 = Some(value);
    }
}

/// Advance all devices that have a changing internal state. Should be called between simulation
/// steps – by convention, we call it immediately after a step.
fn advance_devices(
    q_devices: Query<&mut Device>,
) {
    for mut device in q_devices {
        match *device {
            Device::Counter { value, output_port } => {
                *device = Device::Counter { value: value + 1, output_port }
            }
            // No other devices have an updating internal state
            _ => continue,
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

fn advance_step_number(
    mut simulation_step: ResMut<SimulationStep>,
) {
    simulation_step.0 += 1;
}

fn update_step_number_display(
    simulation_step: Res<SimulationStep>,
    mut q_text: Query<&mut Text, With<SimulationStepLabel>>,
) {
    for mut text in q_text.iter_mut() {
        text.0 = format!("Step {}", simulation_step.0);
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
