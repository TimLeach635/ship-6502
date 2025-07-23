use bevy::prelude::*;
use crate::puzzles::devices::SpawnDeviceCommandExt;
use crate::puzzles::simulation::OutgoingConnection;
use crate::ui::buttons::{ButtonSelected, SpawnButtonCommandExt};

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
enum ItemKind {
    Circle,
    Square,
    EmptyDevice,
}

#[derive(Component)]
struct HasItemKind(ItemKind);

struct ItemSpecification {
    kind: ItemKind,
    name: String,
    button_hue: f32,
}

#[derive(Resource)]
struct CurrentlyPlacing(ItemKind);

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum ItemPlacementState {
    #[default]
    NotPlacing,  // TODO: Rename this option?
    Placing,
    Connecting,
}

#[derive(Resource)]
struct ConnectionOrigin(Option<Entity>);

#[derive(Component)]
struct ItemPlacementField;

pub struct ItemPlacementPlugin;

impl Plugin for ItemPlacementPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ItemPlacementState>();
        app.add_systems(Startup, setup);
        app.add_event::<SelectItem>();
        app.add_event::<DeselectItem>();
        app.add_systems(PostUpdate, display_port_connections
            .after(TransformSystem::TransformPropagate));  // Uses GlobalTransforms

        #[cfg(debug_assertions)]
        {
            app.add_systems(Startup, debug_setup);
            app.add_systems(Update, debug_indicators);
        }
    }
}

fn on_click_connect(
    click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    placement_state: Res<State<ItemPlacementState>>,
    mut next_placement_state: ResMut<NextState<ItemPlacementState>>,
    mut connection_origin: ResMut<ConnectionOrigin>,
) {
    println!("Running `on_click_connect` system");
    // TODO: Enforce that the connection endpoints must be the correct types of port
    match placement_state.get() {
        ItemPlacementState::NotPlacing => {
            connection_origin.0 = Some(click.target);
            next_placement_state.set(ItemPlacementState::Connecting);
        }
        ItemPlacementState::Connecting => {
            // Create connection
            // TODO: Check which end is which!
            //  Currently this assumes that you click the output port first, then the input port
            commands
                .entity(connection_origin.0
                    .expect("Connection origin should be `Some` if in the `Connecting` state"))
                .add_one_related::<OutgoingConnection>(click.target);

            connection_origin.0 = None;
            next_placement_state.set(ItemPlacementState::NotPlacing);
        }
        _ => {/* Don't do anything */}
    }
}

fn display_port_connections(
    mut gizmos: Gizmos,
    q_outgoings: Query<(Entity, &OutgoingConnection)>,
    q_global_transforms: Query<&GlobalTransform>,
) {
    for (source_ent, OutgoingConnection(dest_ent)) in q_outgoings.iter() {
        let source = q_global_transforms.get(source_ent).unwrap();
        let destination = q_global_transforms.get(*dest_ent).unwrap();
        gizmos.arrow_2d(
            source.translation().truncate(),
            destination.translation().truncate(),
            Color::WHITE,
        );
    }
}

#[derive(Event)]
struct SelectItem;

#[derive(Event)]
struct DeselectItem;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Generate list (and hashmap) of placeable items
    let items: Vec<ItemSpecification> = vec![
        ItemSpecification {
            kind: ItemKind::Circle,
            name: "Circle".to_owned(),
            button_hue: 0.0,
        },
        ItemSpecification {
            kind: ItemKind::Square,
            name: "Square".to_owned(),
            button_hue: 240.0,
        },
        ItemSpecification {
            kind: ItemKind::EmptyDevice,
            name: "Empty device".to_owned(),
            button_hue: 160.0,
        },
    ];
    // Ensure the resources are initialised
    commands.insert_resource(CurrentlyPlacing(items[0].kind));
    commands.insert_resource(ConnectionOrigin(None));

    let button_parent = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            align_items: AlignItems::End,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )).id();

    for item in items {
        // Add button
        let button = commands.spawn_button(
            item.button_hue,
            &item.name,
            on_button_click,
        );
        commands.entity(button).insert(HasItemKind(item.kind));
        commands.entity(button_parent).add_child(button);
    }

    // Field on which the items are to be placed
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1000.0, 1000.0))),
        MeshMaterial2d(materials.add(Color::hsl(30.0, 1.0, 0.3))),
    )).observe(place_item_on_click);
}

fn place_item_on_click(
    click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    currently_placing: Res<CurrentlyPlacing>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    placement_state: Res<State<ItemPlacementState>>,
) {
    if *placement_state.get() != ItemPlacementState::Placing {
        return;
    }

    let item = match currently_placing.0 {
        ItemKind::Circle => commands.spawn((
            Mesh2d(meshes.add(Circle::new(50.0))),
            MeshMaterial2d(materials.add(Color::hsl(0.0, 0.95, 0.7))),
        )).id(),
        ItemKind::Square => commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(100.0, 100.0))),
            MeshMaterial2d(materials.add(Color::hsl(240.0, 0.95, 0.7))),
        )).id(),
        ItemKind::EmptyDevice => {
            let entities = commands.spawn_device(
                2,
                2,
                meshes.into_inner(),
                materials.into_inner()
            );

            // Add connection observers on ports
            for ent in entities.input_ports.iter().chain(entities.output_ports.iter()) {
                commands.entity(*ent).observe(on_click_connect);
            }

            entities.base
        },
    };

    // TODO: This positioning works for 2D but should be changed for 3D
    let world_position = click.hit.position
        .expect("Mesh picking hit should be a location in the world");
    commands.entity(item).insert(
        Transform::from_xyz(world_position.x, world_position.y, world_position.z + 1.0),
    );
}

fn on_button_click(
    mut click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    query: Query<(Entity, &HasItemKind)>,
    mut ev_select: EventWriter<SelectItem>,
    mut ev_deselect: EventWriter<DeselectItem>,
    mut currently_placing: ResMut<CurrentlyPlacing>,
    placement_state: Res<State<ItemPlacementState>>,
    mut next_placement_state: ResMut<NextState<ItemPlacementState>>,
) {
    for (entity, HasItemKind(item_kind)) in query {
        if entity == click.target() {
            if placement_state.get() == &ItemPlacementState::Placing
                && currently_placing.0 == *item_kind
            {
                ev_deselect.write(DeselectItem);
                // TODO: Should this be done separately, as a response to the above event?
                next_placement_state.set(ItemPlacementState::NotPlacing);
                commands.entity(entity).remove::<ButtonSelected>();
            } else {
                ev_select.write(SelectItem);
                // TODO: As above, should the following be done elsewhere in response to this event?
                currently_placing.0 = *item_kind;
                next_placement_state.set(ItemPlacementState::Placing);
                commands.entity(entity).insert(ButtonSelected);
            }
        } else {
            // When a button is clicked, all other buttons should deselect
            commands.entity(entity).remove::<ButtonSelected>();
        }
    }

    click.propagate(false);
}

// For debugging
#[cfg(debug_assertions)]
#[derive(Component)]
struct Indicator;

#[cfg(debug_assertions)]
fn debug_setup(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        Indicator,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
    commands.add_observer(debug_all_clicks);
}

#[cfg(debug_assertions)]
fn debug_indicators(
    mut query: Query<&mut Text, With<Indicator>>,
    state: Res<State<ItemPlacementState>>,
    placing: Res<CurrentlyPlacing>,
) {
    for mut text in query.iter_mut() {
        text.0 = format!(
            "{}\n{}",
            match state.get() {
                ItemPlacementState::NotPlacing => "Not placing",
                ItemPlacementState::Placing => "Placing",
                ItemPlacementState::Connecting => "Connecting",
            },
            match placing.0 {
                ItemKind::Circle => "Placing: Circle",
                ItemKind::Square => "Placing: Square",
                ItemKind::EmptyDevice => "Placing: Empty device",
            },
        );
    }
}

#[cfg(debug_assertions)]
fn debug_all_clicks(
    click: Trigger<Pointer<Click>>,
) {
    info!("Target: {:?}", click.target);
    info!("Target(): {:?}", click.target());
}
