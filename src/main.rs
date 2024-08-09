mod computer;

use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use computer::ComputerPlugin;

fn setup_time(mut time: ResMut<Time>) {
    time.set_wrap_period(Duration::from_secs_f32(2.0 * PI));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ComputerPlugin)
        .add_systems(Startup, setup_time)
        .run();
}
