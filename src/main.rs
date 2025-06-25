mod simulation;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    
    let circle = meshes.add(Circle::new(50.0));
    let colour = Color::hsl(1.0, 0.95, 0.7);
    
    commands.spawn((
        Mesh2d(circle),
        MeshMaterial2d(materials.add(colour)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    commands.spawn((
        Text::new("Hello, world!"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}
