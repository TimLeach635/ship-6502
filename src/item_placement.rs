use std::collections::HashMap;
use bevy::prelude::*;
use crate::simulation::{Device, DeviceKind, OutgoingConnection, OutputPortOf, Port};

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
enum ItemKind {
    Circle,
    Square,
    EmptyDevice,
}

#[derive(Component)]
struct HasItemKind(ItemKind);

struct ButtonStateColours {
    background: Color,
    border: Color,
}

struct ButtonColours {
    unselected: ButtonStateColours,
    selected: ButtonStateColours,
    hovered_unselected: ButtonStateColours,
    hovered_selected: ButtonStateColours,
    pressed: ButtonStateColours,
}

impl ButtonColours {
    fn from_hue(hue: f32) -> Self {
        ButtonColours {
            unselected: ButtonStateColours {
                background: Color::hsl(hue, 0.0, 0.15),
                border: Color::BLACK,
            },
            selected: ButtonStateColours {
                background: Color::hsl(hue, 0.8, 0.20),
                border: Color::BLACK,
            },
            hovered_unselected: ButtonStateColours {
                background: Color::hsl(hue, 0.0, 0.25),
                border: Color::WHITE,
            },
            hovered_selected: ButtonStateColours {
                background: Color::hsl(hue, 0.8, 0.25),
                border: Color::WHITE,
            },
            pressed: ButtonStateColours {
                background: Color::hsl(hue, 0.8, 0.40),
                border: Color::hsl(hue, 1.0, 1.0),
            },
        }
    }
}

struct ItemSpecification {
    kind: ItemKind,
    name: String,
    button_colours: ButtonColours,
}

#[derive(Resource)]
struct CurrentlyPlacing(ItemKind);

#[derive(Resource)]
struct PlaceableItems(HashMap<ItemKind, ItemSpecification>);

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
        app.add_systems(Update, (
            button_visuals_system,
            update_button_colour,
        ));
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
struct SelectItem(ItemKind);

#[derive(Event)]
struct DeselectItem;

/// The system for managing the visuals of the buttons.
/// This is done separately to the item switching logic.
fn button_visuals_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &HasItemKind,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    currently_placing: Res<CurrentlyPlacing>,
    placeable_items: Res<PlaceableItems>,
    placement_state: Res<State<ItemPlacementState>>,
) {
    for (
        interaction,
        mut color,
        mut border_color,
        HasItemKind(item_kind),
    ) in &mut interaction_query {
        let item_specification = &placeable_items.0[item_kind];

        if placement_state.get() == &ItemPlacementState::Placing
            && currently_placing.0 == *item_kind {
            match *interaction {
                Interaction::Pressed => {
                    color.0 = item_specification.button_colours.pressed.background;
                    border_color.0 = item_specification.button_colours.pressed.border;
                }
                Interaction::Hovered => {
                    color.0 = item_specification.button_colours.hovered_selected.background;
                    border_color.0 = item_specification.button_colours.hovered_selected.border;
                }
                Interaction::None => {
                    color.0 = item_specification.button_colours.selected.background;
                    border_color.0 = item_specification.button_colours.selected.border;
                }
            }
        } else {
            match *interaction {
                Interaction::Pressed => {
                    color.0 = item_specification.button_colours.pressed.background;
                    border_color.0 = item_specification.button_colours.pressed.border;
                }
                Interaction::Hovered => {
                    color.0 = item_specification.button_colours.hovered_unselected.background;
                    border_color.0 = item_specification.button_colours.hovered_unselected.border;
                }
                Interaction::None => {
                    color.0 = item_specification.button_colours.unselected.background;
                    border_color.0 = item_specification.button_colours.unselected.border;
                }
            }
        }
    }
}

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
            button_colours: ButtonColours::from_hue(0.0),
        },
        ItemSpecification {
            kind: ItemKind::Square,
            name: "Square".to_owned(),
            button_colours: ButtonColours::from_hue(240.0),
        },
        ItemSpecification {
            kind: ItemKind::EmptyDevice,
            name: "Empty device".to_owned(),
            button_colours: ButtonColours::from_hue(160.0),
        },
    ];
    // Ensure the resources are initialised
    commands.insert_resource(CurrentlyPlacing(items[0].kind));
    commands.insert_resource(ConnectionOrigin(None));

    let mut item_map: HashMap<ItemKind, ItemSpecification> = HashMap::new();

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
        let button = commands.spawn((
            Button,
            HasItemKind(item.kind),
            Node {
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(item.button_colours.unselected.border),
            BorderRadius::MAX,
            BackgroundColor(item.button_colours.unselected.background),
            children![(
                Text::new(item.name.clone()),
                TextColor(Color::WHITE),
                TextShadow::default(),
            )]
        )).observe(on_button_click).id();
        commands.entity(button_parent).add_child(button);

        // Add to map
        if item_map.insert(item.kind, item).is_some() {
            warn!("Duplicate ItemKind found in list of items");
        }
    }
    commands.insert_resource(PlaceableItems(item_map));

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
            let device = commands.spawn((
                Device { kind: DeviceKind::Empty },
                Mesh2d(meshes.add(Rectangle::new(80.0, 100.0))),
                MeshMaterial2d(materials.add(Color::hsl(160.0, 0.95, 0.7))),
            )).id();

            let output_port_1 = commands.spawn((
                Port(None),
                Mesh2d(meshes.add(Rectangle::new(15.0, 30.0))),
                MeshMaterial2d(materials.add(Color::hsl(20.0, 0.95, 0.7))),
                Transform::from_xyz(40.0, 30.0, 0.0),
            )).observe(on_click_connect).id();
            let output_port_2 = commands.spawn((
                Port(None),
                Mesh2d(meshes.add(Rectangle::new(15.0, 30.0))),
                MeshMaterial2d(materials.add(Color::hsl(20.0, 0.95, 0.7))),
                Transform::from_xyz(40.0, -30.0, 0.0),
            )).observe(on_click_connect).id();
            let input_port_1 = commands.spawn((
                Port(None),
                Mesh2d(meshes.add(Rectangle::new(15.0, 30.0))),
                MeshMaterial2d(materials.add(Color::hsl(340.0, 0.95, 0.7))),
                Transform::from_xyz(-40.0, 30.0, 0.0),
            )).observe(on_click_connect).id();
            let input_port_2 = commands.spawn((
                Port(None),
                Mesh2d(meshes.add(Rectangle::new(15.0, 30.0))),
                MeshMaterial2d(materials.add(Color::hsl(340.0, 0.95, 0.7))),
                Transform::from_xyz(-40.0, -30.0, 0.0),
            )).observe(on_click_connect).id();

            commands.entity(device).add_children(&vec![
                output_port_1,
                output_port_2,
                input_port_1,
                input_port_2,
            ]);
            commands.entity(output_port_1).add_one_related::<OutputPortOf>(device);
            commands.entity(output_port_2).add_one_related::<OutputPortOf>(device);
            commands.entity(input_port_1).add_one_related::<OutputPortOf>(device);
            commands.entity(input_port_2).add_one_related::<OutputPortOf>(device);

            device
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
    query: Query<&HasItemKind>,
    mut ev_select: EventWriter<SelectItem>,
    mut ev_deselect: EventWriter<DeselectItem>,
    mut currently_placing: ResMut<CurrentlyPlacing>,
    placement_state: Res<State<ItemPlacementState>>,
    mut next_placement_state: ResMut<NextState<ItemPlacementState>>,
) {
    let HasItemKind(item_kind) = query.get(click.target())
        .expect("This observer should not trigger unless the target is an entity with a HasItemKind component");
    if placement_state.get() == &ItemPlacementState::Placing
        && currently_placing.0 == *item_kind
    {
        ev_deselect.write(DeselectItem);
        // TODO: Should this be done separately, as a response to the above event?
        next_placement_state.set(ItemPlacementState::NotPlacing);
    } else {
        ev_select.write(SelectItem(*item_kind));
        // TODO: As above, should the following be done elsewhere in response to this event?
        currently_placing.0 = *item_kind;
        next_placement_state.set(ItemPlacementState::Placing);
    }
    click.propagate(false);
}

// TODO: There's gotta be a way of combining this with the identical code above, and
//  I WILL FIND IT
fn update_button_colour(
    mut query: Query<(
        &Interaction,
        &mut BackgroundColor,
        &mut BorderColor,
        &HasItemKind,
    )>,
    mut ev_select: EventReader<SelectItem>,
    mut ev_deselect: EventReader<DeselectItem>,
    currently_placing: Res<CurrentlyPlacing>,
    placeable_items: Res<PlaceableItems>,
    placement_state: Res<State<ItemPlacementState>>,
) {
    // I'm not sure how I feel about doing it like this. Is this inefficient?
    // I'm putting this note here because I'm a notorious premature optimiser, so if I
    // don't address it in some way I will explode

    // This is gross, but we run this update on all the buttons if there are any select
    // OR deselect events, because it just doesn't matter otherwise
    if ev_select.read().count() > 0 || ev_deselect.read().count() > 0 {
        for (
            interaction,
            mut color,
            mut border_color,
            HasItemKind(item_kind),
        ) in query.iter_mut() {
            let item_specification = &placeable_items.0[item_kind];

            if placement_state.get() == &ItemPlacementState::Placing
                && currently_placing.0 == *item_kind {
                match *interaction {
                    Interaction::Pressed => {
                        color.0 = item_specification.button_colours.pressed.background;
                        border_color.0 = item_specification.button_colours.pressed.border;
                    }
                    Interaction::Hovered => {
                        color.0 = item_specification.button_colours.hovered_selected.background;
                        border_color.0 = item_specification.button_colours.hovered_selected.border;
                    }
                    Interaction::None => {
                        color.0 = item_specification.button_colours.selected.background;
                        border_color.0 = item_specification.button_colours.selected.border;
                    }
                }
            } else {
                match *interaction {
                    Interaction::Pressed => {
                        color.0 = item_specification.button_colours.pressed.background;
                        border_color.0 = item_specification.button_colours.pressed.border;
                    }
                    Interaction::Hovered => {
                        color.0 = item_specification.button_colours.hovered_unselected.background;
                        border_color.0 = item_specification.button_colours.hovered_unselected.border;
                    }
                    Interaction::None => {
                        color.0 = item_specification.button_colours.unselected.background;
                        border_color.0 = item_specification.button_colours.unselected.border;
                    }
                }
            }
        }
    }
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
