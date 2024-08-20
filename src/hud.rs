use bevy::prelude::*;

pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    // UI camera
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1,  // after the main camera
                ..default()
            },
            ..default()
        },
        IsDefaultUiCamera,
    ));

    // Dot in centre of screen, to act as a crosshair
    // No, I'm under no illusions that this looks remotely good
    // Yes, I'll change it
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(2.0),
                    height: Val::Px(2.0),
                    ..default()
                },
                background_color: Color::WHITE.into(),
                ..default()
            });
        });
}
