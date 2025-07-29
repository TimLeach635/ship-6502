use crate::puzzles::item_placement::{AcceptsConnectionStart, AcceptsConnectionEnd};
use crate::puzzles::simulation::Port;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use serde::Deserialize;

pub struct PuzzlePlugin;

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (
            setup,
            create_ports_for_puzzle.after(setup),
        ));
    }
}

#[derive(Component)]
#[require(Port, AcceptsConnectionStart)]
struct PuzzleInput {
    name: String,
}

#[derive(Component)]
#[require(Port, AcceptsConnectionEnd)]
struct PuzzleOutput {
    name: String,
}

/// Represents either an input or an output of a puzzle.
///
/// If it's an input, stores the provided values at each timestep.
///
/// If an output, the _expected_ values at each timestep.
#[derive(Debug, Deserialize)]
struct PuzzleInterfaceSpec {
    name: String,
    values: Vec<u32>,
}

#[derive(Debug, Deserialize)]
pub struct PuzzleSpec {
    inputs: Vec<PuzzleInterfaceSpec>,
    outputs: Vec<PuzzleInterfaceSpec>,
}

#[derive(Resource)]
pub struct Puzzle(PuzzleSpec);

fn setup(mut commands: Commands) {
    // Load puzzle from file
    // TODO: Multiple puzzles
    let puzzle_file = std::fs::File::open("puzzle.json")
        .expect("Should be able to open puzzle.json");
    let puzzle: PuzzleSpec = serde_json::from_reader(puzzle_file)
        .expect("Should be able to parse puzzle.json as a PuzzleSpec");
    commands.insert_resource(Puzzle(puzzle));
}

fn create_ports_for_puzzle(
    mut commands: Commands,
    puzzle: Res<Puzzle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let inputs_x = -350.0;
    let outputs_x = 350.0;

    let inputs_start_y = -200.0;
    let outputs_start_y = 200.0;

    let mesh = meshes.add(Circle::new(20.0));

    let input_material = materials.add(Color::hsl(240.0, 0.95, 0.7));
    let output_material = materials.add(Color::hsl(170.0, 0.95, 0.7));

    for (idx, input) in puzzle.0.inputs.iter().enumerate() {
        commands.spawn((
            PuzzleInput { name: input.name.clone() },
            Transform::from_xyz(inputs_x, inputs_start_y + 50.0 * (idx as f32), 1.0),
            Mesh2d(mesh.clone()),
            MeshMaterial2d(input_material.clone()),
            children![(
                Text2d(input.name.clone()),
                TextColor(Color::BLACK),
                Anchor::CenterLeft,
                Transform::from_xyz(25.0, 0.0, 1.0),
            )],
        ));
    }

    for (idx, output) in puzzle.0.outputs.iter().enumerate() {
        commands.spawn((
            PuzzleOutput { name: output.name.clone() },
            Transform::from_xyz(outputs_x, outputs_start_y - 50.0 * (idx as f32), 1.0),
            Mesh2d(mesh.clone()),
            MeshMaterial2d(output_material.clone()),
            children![(
                Text2d(output.name.clone()),
                TextColor(Color::BLACK),
                Anchor::CenterLeft,
                Transform::from_xyz(25.0, 0.0, 1.0),
            )],
        ));
    }
}
