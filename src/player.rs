use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player);
        app.add_systems(Update, (camera_mouse_capturing, camera_looking));
    }
}

#[derive(Component)]
pub struct Player {
    horiz_look_sensitivity: f32,
    vert_look_sensitivity: f32,
}

fn setup_player(
    mut commands: Commands,
) {
    // Spawn player
    let player = commands.spawn((
        Player {
            horiz_look_sensitivity: 0.7,
            vert_look_sensitivity: 0.7,
        },
        TransformBundle::default(),
    )).id();

    // Spawn camera
    let camera = commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 1.5, 0.0).looking_at(Vec3::new(0.0, 1.5, -1.0), Vec3::Y),
        ..default()
    }).id();

    // Set the camera to be the child of the player
    commands.entity(player).push_children(&[camera]);
}

fn camera_mouse_capturing(
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
) {
    let mut window = windows.single_mut();

    // Capture mouse on click
    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    // Release mouse on escape
    if key.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}

fn camera_looking(
    time: Res<Time>,
    mut evr_mouse: EventReader<MouseMotion>,
    windows: Query<&Window>,
    mut players: Query<(&mut Transform, &Player, &Children), Without<Camera>>,
    mut cameras: Query<&mut Transform, With<Camera>>,
) {
    let window = windows.single();
    if window.cursor.grab_mode == CursorGrabMode::Locked {
        let (mut player_transform, player, children) = players.single_mut();
        for &child in children.iter() {
            let mut camera_transform = match cameras.get_mut(child) {
                Ok(result) => result,
                Err(_) => break,
            };

            for event in evr_mouse.read() {
                let dx = event.delta.x * player.horiz_look_sensitivity * time.delta_seconds();
                let dy = event.delta.y * player.vert_look_sensitivity * time.delta_seconds();
    
                player_transform.rotate_y(-dx);
                camera_transform.rotate_x(-dy);
            }
        }
    }
}
