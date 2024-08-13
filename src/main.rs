mod computer;
mod player;

use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use computer::ComputerPlugin;
use player::PlayerPlugin;

fn setup_time(mut time: ResMut<Time>) {
    time.set_wrap_period(Duration::from_secs_f32(2.0 * PI));
}

// Add a checkerboard surface for testing visual stuff
fn add_checkerboard(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let plane_mesh = meshes.add(Plane3d::default().mesh().size(0.5, 0.5));
    let black_mat = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        reflectance: 0.3,
        perceptual_roughness: 0.8,
        ..default()
    });
    let white_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        reflectance: 0.3,
        perceptual_roughness: 0.8,
        ..default()
    });

    for x in -6..=6 {
        for z in -12..=6 {
            commands.spawn(PbrBundle {
                mesh: plane_mesh.clone(),
                material: if (x + z) % 2 == 0 {
                    black_mat.clone()
                } else {
                    white_mat.clone()
                },
                transform: Transform::from_xyz(x as f32 * 0.5, 0.0, z as f32 * 0.5),
                ..default()
            });
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayerPlugin)
        .add_plugins(ComputerPlugin)
        .add_systems(Startup, (setup_time, add_checkerboard))
        .run();
}
