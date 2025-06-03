use std::collections::HashMap;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_systems(Startup, test_startup)
        .add_systems(Update, (
            resolve,
            (
                reset_input_ports,
                reset_output_ports,
            ).after(resolve),
        ))
        .run();
}

enum DeviceKind {
    Empty,
}

enum ResolutionState {
    Unresolved,
    Resolved(u32),
}

#[derive(Component)]
struct Device {
    kind: DeviceKind,
}

#[derive(Component)]
/// A device that outputs information we want to know.
/// It will be evaluated first.
struct OutputDevice;

#[derive(Component)]
struct InputPort {
    state: ResolutionState,
}

#[derive(Component)]
#[relationship(relationship_target = Inputs)]
struct InputOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = InputOf)]
struct Inputs(Vec<Entity>);

#[derive(Component)]
struct OutputPort {
    state: ResolutionState,
}

#[derive(Component)]
#[relationship(relationship_target = Outputs)]
struct OutputOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = OutputOf)]
struct Outputs(Vec<Entity>);

#[derive(Component)]
struct Name(String);

#[derive(Component)]
#[relationship(relationship_target = OutgoingConnections)]
struct IncomingConnection(Entity);

#[derive(Component)]
#[relationship_target(relationship = IncomingConnection)]
struct OutgoingConnections(Vec<Entity>);

fn test_startup(mut commands: Commands) {
    // Devices
    let constant_device = commands.spawn((
        Device {
            kind: DeviceKind::Empty,
        },
        OutputDevice,
        Name("Empty".to_string()),
    )).id();
}

fn resolve(
    q_devices: Query<(&Device, &Name), With<OutputDevice>>,
) {
    for (device, name) in q_devices.iter() {
        match device.kind {
            DeviceKind::Empty => {
                println!("Device \"{}\" resolved", name.0);
            }
        }
    }
}

fn reset_input_ports(mut ports: Query<&mut InputPort>) {
    for mut device in ports.iter_mut() {
        device.state = ResolutionState::Unresolved;
    }
}

fn reset_output_ports(mut ports: Query<&mut OutputPort>) {
    for mut device in ports.iter_mut() {
        device.state = ResolutionState::Unresolved;
    }
}
