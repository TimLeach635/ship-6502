mod simulation;
mod item_placement;

use bevy::prelude::*;
use crate::item_placement::ItemPlacementPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ItemPlacementPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(Camera2d);
}
