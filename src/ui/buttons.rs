use bevy::color::Color;
use bevy::ecs::system::IntoObserverSystem;
use bevy::prelude::*;

#[derive(Component)]
pub struct ButtonSelected;

#[derive(Component)]
struct HasColours(ButtonColours);

struct ButtonStateColours {
    background: Color,
    border: Color,
}

struct ButtonColours {
    unselected: ButtonStateColours,
    selected: ButtonStateColours,
    hovered_unselected: ButtonStateColours,
    hovered_selected: ButtonStateColours,
    pressed: ButtonStateColours,
}

impl ButtonColours {
    fn from_hue(hue: f32) -> Self {
        ButtonColours {
            unselected: ButtonStateColours {
                background: Color::hsl(hue, 0.0, 0.15),
                border: Color::BLACK,
            },
            selected: ButtonStateColours {
                background: Color::hsl(hue, 0.8, 0.20),
                border: Color::BLACK,
            },
            hovered_unselected: ButtonStateColours {
                background: Color::hsl(hue, 0.0, 0.25),
                border: Color::WHITE,
            },
            hovered_selected: ButtonStateColours {
                background: Color::hsl(hue, 0.8, 0.25),
                border: Color::WHITE,
            },
            pressed: ButtonStateColours {
                background: Color::hsl(hue, 0.8, 0.40),
                border: Color::hsl(hue, 1.0, 1.0),
            },
        }
    }
}

pub trait SpawnButtonCommandExt<B: Bundle, M> {
    fn spawn_button(
        &mut self,
        hue: f32,
        text: &str,
        on_click: impl IntoObserverSystem<Pointer<Click>, B, M>
    ) -> Entity;
}

impl<'w, 's, B: Bundle, M> SpawnButtonCommandExt<B, M> for Commands<'w, 's> {
    fn spawn_button(
        &mut self,
        hue: f32,
        text: &str,
        on_click: impl IntoObserverSystem<Pointer<Click>, B, M>
    ) -> Entity {
        self.spawn((
            Button,
            Node {
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            HasColours(ButtonColours::from_hue(hue)),
            BorderColor(Color::BLACK),
            BorderRadius::MAX,
            BackgroundColor(Color::hsl(hue, 0.0, 0.15)),
            children![(
                Text::new(text.to_owned()),
                TextColor(Color::WHITE),
                TextShadow::default(),
            )]
        )).observe(on_click).id()
    }
}

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_button_colours);
    }
}

// TODO: Figure out how to stop this from running every frame
fn update_button_colours(
    mut query: Query<(
        &Interaction,
        &mut BackgroundColor,
        &mut BorderColor,
        &HasColours,
        Option<&ButtonSelected>,
    )>,
) {
    for (
        interaction,
        mut color,
        mut border_color,
        HasColours(button_colours),
        opt_selected,
    ) in &mut query {
        if opt_selected.is_some() {
            match *interaction {
                Interaction::Pressed => {
                    color.0 = button_colours.pressed.background;
                    border_color.0 = button_colours.pressed.border;
                }
                Interaction::Hovered => {
                    color.0 = button_colours.hovered_selected.background;
                    border_color.0 = button_colours.hovered_selected.border;
                }
                Interaction::None => {
                    color.0 = button_colours.selected.background;
                    border_color.0 = button_colours.selected.border;
                }
            }
        } else {
            match *interaction {
                Interaction::Pressed => {
                    color.0 = button_colours.pressed.background;
                    border_color.0 = button_colours.pressed.border;
                }
                Interaction::Hovered => {
                    color.0 = button_colours.hovered_unselected.background;
                    border_color.0 = button_colours.hovered_unselected.border;
                }
                Interaction::None => {
                    color.0 = button_colours.unselected.background;
                    border_color.0 = button_colours.unselected.border;
                }
            }
        }
    }
}
