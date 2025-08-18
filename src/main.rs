mod puzzles;
mod ui;

use bevy::prelude::*;
use crate::puzzles::*;
use crate::ui::buttons::ButtonPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshPickingPlugin,
            ItemPlacementPlugin,
            ButtonPlugin,
            SimulationPlugin,
            PuzzlePlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(Camera2d);
}
