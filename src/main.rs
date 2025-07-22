mod puzzles;

use bevy::prelude::*;
use crate::puzzles::item_placement::ItemPlacementPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin, ItemPlacementPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(Camera2d);
}
