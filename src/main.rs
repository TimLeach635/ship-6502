use std::cmp::PartialEq;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_systems(Startup, test_startup)
        .add_systems(Update, (
            resolve,
            reset_resolutions.after(resolve),
        ))
        .run();
}

enum DeviceKind {
    Empty,
}

#[derive(PartialEq)]
enum ResolutionState {
    Unresolved,
    Resolved(u32),
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
#[relationship(relationship_target = OutgoingConnections)]
struct IncomingConnection(Entity);

#[derive(Component)]
#[relationship_target(relationship = IncomingConnection)]
struct OutgoingConnections(Vec<Entity>);

fn test_startup(mut commands: Commands) {
    // Devices
    commands.spawn((
        Device {
            kind: DeviceKind::Empty,
        },
        Name("No inputs 1".to_string()),
    ));
    commands.spawn((
        Device {
            kind: DeviceKind::Empty,
        },
        Name("No inputs 2".to_string()),
    ));
    commands
        .spawn((
            Device {
                kind: DeviceKind::Empty,
            },
            Name("Has inputs 1".to_string()),
        )).with_related_entities::<InputPortOf>(|spawner| {
            spawner.spawn(Name("Input port 1.1".to_owned()));
            spawner.spawn(Name("Input port 1.2".to_owned()));
        });
    commands
        .spawn((
            Device {
                kind: DeviceKind::Empty,
            },
            Name("Has inputs 2".to_string()),
        )).with_related::<InputPortOf>(Name("Input port 2.1".to_owned()));
}

fn resolve(
    q_root_devices: Query<(Entity, &Name), (With<Device>, Without<InputPorts>)>,
) {
    // Start at the devices that don't have any input ports, as it is these devices
    //  whose behaviour is known right away
    for (ent, name) in q_root_devices {
        println!("Identified \"{}\" as a root node", name.0);
    }
}

fn reset_resolutions(q_resolvable: Query<&mut Resolvable>) {
    for mut resolvable in q_resolvable {
        resolvable.0 = ResolutionState::Unresolved;
    }
}
