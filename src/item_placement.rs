use bevy::{prelude::*, color::palettes::basic::*};
use bevy::input::mouse::MouseButtonInput;

// Colours
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub struct ItemPlacementPlugin;

struct PlaceableItemBundle {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

#[derive(Resource)]
struct CurrentlyPlacing(PlaceableItemBundle);

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum ItemPlacementState {
    #[default]
    NotPlacing,
    Placing,
}

impl Plugin for ItemPlacementPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ItemPlacementState>();
        app.add_systems(Startup, setup);
        app.add_systems(Update, (placement_system, button_system));
    }
}

fn placement_system(
    mut commands: Commands,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    currently_placing: Res<CurrentlyPlacing>,
    q_window: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    // Assume exactly one camera
    // TODO: Don't assume that
    let (camera, camera_transform) = q_camera
        .single()
        .expect("Should be only one camera object");
    
    for event in mouse_button_input_events.read() {
        // Figure out where the mouse click was
        let window = q_window
            .get(event.window)
            .expect("This entity should always be a window, and therefore always be in this query");
        
        let cursor_position = window.cursor_position()
            .expect("The cursor should be within the bounds of the window this event fired on");
        
        let world_position = camera.viewport_to_world_2d(camera_transform, cursor_position)
            .expect("Should not experience a viewport conversion error here");
        
        commands.spawn((
            // These are handles, so we can clone them at negligible cost
            Mesh2d(currently_placing.0.mesh.clone()),
            MeshMaterial2d(currently_placing.0.material.clone()),
            Transform::from_xyz(world_position.x, world_position.y, 0.),
        ));
    }
}

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::End,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Button,
            Node {
                width: Val::Px(150.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::MAX,
            BackgroundColor(NORMAL_BUTTON),
            children![(
                Text::new("Circle"),
                TextColor(Color::WHITE),
                TextShadow::default(),
            )]
        ), (
            Button,
            Node {
                width: Val::Px(150.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::MAX,
            BackgroundColor(NORMAL_BUTTON),
            children![(
                Text::new("Square"),
                TextColor(Color::WHITE),
                TextShadow::default(),
            )]
        )]
    ));
    
    commands.insert_resource(CurrentlyPlacing(PlaceableItemBundle {
        mesh: meshes.add(Circle::new(50.0)),
        material: materials.add(Color::hsl(1.0, 0.95, 0.7)),
    }));
}
