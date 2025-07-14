use std::collections::HashMap;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::input::mouse::MouseButtonInput;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
enum ItemKind {
    Circle,
    Square,
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
    NotPlacing,
    Placing,
}

pub struct ItemPlacementPlugin;

impl Plugin for ItemPlacementPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ItemPlacementState>();
        app.add_systems(Startup, setup);
        app.add_systems(Update, (
            placement_system.run_if(in_state(ItemPlacementState::Placing)),
            button_system
        ));
    }
}

fn placement_system(
    mut commands: Commands,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    currently_placing: Res<CurrentlyPlacing>,
    q_window: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Assume exactly one camera
    // TODO: Don't assume that
    let (camera, camera_transform) = q_camera
        .single()
        .expect("Should be only one camera object");
    
    for event in mouse_button_input_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        // Figure out where the mouse click was
        let window = q_window
            .get(event.window)
            .expect("This entity should always be a window, and therefore always be in this query");
        
        let cursor_position = window.cursor_position()
            .expect("The cursor should be within the bounds of the window this event fired on");
        
        let world_position = camera.viewport_to_world_2d(camera_transform, cursor_position)
            .expect("Should not experience a viewport conversion error here");

        let item = match currently_placing.0 {
            ItemKind::Circle => commands.spawn((
                Mesh2d(meshes.add(Circle::new(50.0))),
                MeshMaterial2d(materials.add(Color::hsl(0.0, 0.95, 0.7))),
            )).id(),
            ItemKind::Square => commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(100.0, 100.0))),
                MeshMaterial2d(materials.add(Color::hsl(240.0, 0.95, 0.7))),
            )).id(),
        };

        commands.entity(item).insert(
            Transform::from_xyz(world_position.x, world_position.y, 0.)
        );
    }
}

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &HasItemKind,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut currently_placing: ResMut<CurrentlyPlacing>,
    placeable_items: Res<PlaceableItems>,
    item_placement_state: Res<State<ItemPlacementState>>,
    mut next_item_placement_state: ResMut<NextState<ItemPlacementState>>,
) {
    for (
        interaction,
        mut color,
        mut border_color,
        HasItemKind(item_kind),
    ) in &mut interaction_query {
        let item_specification = &placeable_items.0[item_kind];

        if item_placement_state.get() == &ItemPlacementState::Placing
            && currently_placing.0 == *item_kind {
            match *interaction {
                Interaction::Pressed => {
                    color.0 = item_specification.button_colours.pressed.background;
                    border_color.0 = item_specification.button_colours.pressed.border;

                    next_item_placement_state.set(ItemPlacementState::NotPlacing);
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

                    // TODO: I don't like that doing the state change logic here means that
                    //  it happens as soon as you press the button. I'd rather that it changes
                    //  when you let go, but the logic is more complicated there.
                    //  See if these buttons fire off events or something that you can listen to
                    //  - it's possible that this logic should only be used for the cosmetic
                    //  changes.
                    currently_placing.0 = *item_kind;
                    next_item_placement_state.set(ItemPlacementState::Placing);
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
    ];
    // Ensure the resource is initialised
    commands.insert_resource(CurrentlyPlacing(items[0].kind));

    let mut item_map: HashMap<ItemKind, ItemSpecification> = HashMap::new();

    let button_parent = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
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
                width: Val::Px(150.0),
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
        )).id();
        commands.entity(button_parent).add_child(button);

        // Add to map
        if item_map.insert(item.kind, item).is_some() {
            warn!("Duplicate ItemKind found in list of items");
        }
    }
    commands.insert_resource(PlaceableItems(item_map));
}
