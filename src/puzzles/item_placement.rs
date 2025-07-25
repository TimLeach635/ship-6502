use bevy::prelude::*;
use crate::puzzles::devices::{DeviceKind, SpawnDeviceCommandExt};
use crate::puzzles::simulation::{ConnectionStart, InputPort, OutputPort};
use crate::ui::buttons::{ButtonSelected, SpawnButtonCommandExt};

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
enum ItemKind {
    ConstantDevice,
    CounterDevice,
    RepeaterDevice,
    AdderDevice,
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

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
enum InputOrOutput {
    #[default]
    Input,
    Output,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum ItemPlacementState {
    #[default]
    NotPlacing,  // TODO: Rename this option?
    Placing,
    Connecting {
        origin: InputOrOutput,
    },
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
    q_ports: Query<(Option<&InputPort>, Option<&OutputPort>), Or<(With<InputPort>, With<OutputPort>)>>,
    placement_state: Res<State<ItemPlacementState>>,
    mut next_placement_state: ResMut<NextState<ItemPlacementState>>,
    mut connection_origin: ResMut<ConnectionOrigin>,
) {
    if let Ok((in_opt, out_opt)) = q_ports.get(click.target()) {
        assert_ne!(
            in_opt.is_some(), out_opt.is_some(),
            "Either this is an input or an output, but it shouldn't be both."
        );
        match placement_state.get() {
            ItemPlacementState::NotPlacing => {
                connection_origin.0 = Some(click.target);
                next_placement_state.set(ItemPlacementState::Connecting {
                    origin: match in_opt.is_some() {
                        true => InputOrOutput::Input,
                        false => InputOrOutput::Output,
                    }
                });
            }
            ItemPlacementState::Connecting { origin } => {
                // Create connection.
                // Can only connect inputs to outputs and vice versa, so:
                match origin {
                    InputOrOutput::Input => {
                        if out_opt.is_some() {
                            commands
                                .entity(connection_origin.0
                                    .expect("Connection origin should be `Some` if in the `Connecting` state"))
                                .add_one_related::<ConnectionStart>(click.target);

                            connection_origin.0 = None;
                            next_placement_state.set(ItemPlacementState::NotPlacing);
                        }
                    }
                    InputOrOutput::Output => {
                        if in_opt.is_some() {
                            commands
                                .entity(click.target)
                                .add_one_related::<ConnectionStart>(connection_origin.0
                                    .expect("Connection origin should be `Some` if in the `Connecting` state"));

                            connection_origin.0 = None;
                            next_placement_state.set(ItemPlacementState::NotPlacing);
                        }
                    }
                }
                // Does nothing if the player clicks on two inputs or two outputs in a row
                // Note that this also forbids the user from connecting a port to itself, which
                // is a nice freebie
            }
            _ => {/* Don't do anything */}
        }
    }
}

fn display_port_connections(
    mut gizmos: Gizmos,
    q_outgoings: Query<(Entity, &ConnectionStart)>,
    q_global_transforms: Query<&GlobalTransform>,
) {
    for (source_ent, ConnectionStart(dest_ent)) in q_outgoings.iter() {
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
            kind: ItemKind::ConstantDevice,
            name: "Constant device".to_owned(),
            button_hue: 0.0,
        },
        ItemSpecification {
            kind: ItemKind::CounterDevice,
            name: "Counter device".to_owned(),
            button_hue: 90.0,
        },
        ItemSpecification {
            kind: ItemKind::RepeaterDevice,
            name: "Repeater device".to_owned(),
            button_hue: 180.0,
        },
        ItemSpecification {
            kind: ItemKind::AdderDevice,
            name: "Adder device".to_owned(),
            button_hue: 270.0,
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
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    placement_state: Res<State<ItemPlacementState>>,
) {
    if *placement_state.get() != ItemPlacementState::Placing {
        return;
    }

    let item = match currently_placing.0 {
        ItemKind::ConstantDevice => {
            let value = 10u32;

            let entities = commands.spawn_device(
                // TODO: Allow the user to choose the value
                DeviceKind::Constant { value },
                meshes.into_inner(),
                materials.into_inner()
            );

            // Add connection observers on ports
            for ent in entities.output_ports.iter() {  // Can omit inputs as there aren't any!
                commands.entity(*ent).observe(on_click_connect);
            }

            // Add text showing value
            let label = commands.spawn((
                Text2d(value.to_string()),
                TextColor(Color::BLACK),
            )).id();
            commands.entity(entities.base).add_child(label);

            entities.base
        },
        ItemKind::CounterDevice => {
            let entities = commands.spawn_device(
                // TODO: Allow the user to choose the value
                DeviceKind::Counter,
                meshes.into_inner(),
                materials.into_inner()
            );

            // Add connection observers on ports
            for ent in entities.output_ports.iter() {  // Can omit inputs as there aren't any!
                commands.entity(*ent).observe(on_click_connect);
            }

            // Add text showing value
            let label = commands.spawn((
                Text2d("Cnt".to_owned()),
                TextColor(Color::BLACK),
            )).id();
            commands.entity(entities.base).add_child(label);

            entities.base
        },
        ItemKind::RepeaterDevice => {
            let entities = commands.spawn_device(
                DeviceKind::Repeater,
                meshes.into_inner(),
                materials.into_inner()
            );

            // Add connection observers on ports
            for ent in entities.input_ports.iter().chain(entities.output_ports.iter()) {
                commands.entity(*ent).observe(on_click_connect);
            }

            // Add text showing value
            let label = commands.spawn((
                Text2d("Rep".to_owned()),
                TextColor(Color::BLACK),
            )).id();
            commands.entity(entities.base).add_child(label);

            entities.base
        },
        ItemKind::AdderDevice => {
            let entities = commands.spawn_device(
                DeviceKind::Adder,
                meshes.into_inner(),
                materials.into_inner()
            );

            // Add connection observers on ports
            for ent in entities.input_ports.iter().chain(entities.output_ports.iter()) {
                commands.entity(*ent).observe(on_click_connect);
            }

            // Add text showing value
            let label = commands.spawn((
                Text2d("Add".to_owned()),
                TextColor(Color::BLACK),
            )).id();
            commands.entity(entities.base).add_child(label);

            entities.base
        },
    };

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
                ItemPlacementState::Connecting { origin: InputOrOutput::Input } => "Connecting (from input)",
                ItemPlacementState::Connecting { origin: InputOrOutput::Output } => "Connecting (from output)",
            },
            match placing.0 {
                ItemKind::ConstantDevice => "Placing: Constant device",
                ItemKind::CounterDevice => "Placing: Counter device",
                ItemKind::RepeaterDevice => "Placing: Repeater device",
                ItemKind::AdderDevice => "Placing: Adder device",
            },
        );
    }
}

#[cfg(debug_assertions)]
fn debug_all_clicks(
    click: Trigger<Pointer<Click>>,
) {
    trace!("Target: {:?}", click.target);
    trace!("Target(): {:?}", click.target());
}
