mod ibm_byte_map;
mod os;
mod ship_os;
mod terminal;

use std::f32::consts::PI;

use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::render::render_resource::TextureDescriptor;
use bevy::render::render_resource::TextureDimension;
use bevy::render::render_resource::TextureFormat;
use bevy::render::render_resource::TextureUsages;
use bevy::render::view::RenderLayers;
use bevy::sprite::Anchor;
use bevy::text::Text;
use bevy::text::Text2dBounds;
use ship_os::ShipOS;

pub struct ComputerPlugin;
impl Plugin for ComputerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_computer);
        app.add_systems(Update, (rotator_system, draw_screen));
    }
}

#[derive(Component)]
struct ScreenCuboid;

fn setup_computer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    // The code in here comes largely from the Bevy "render to texture" example
    // https://github.com/bevyengine/bevy/blob/latest/examples/3d/render_to_texture.rs
    // I did this in an evening while getting slowly more drunk and I'm extremely proud of myself
    // it was really hard
    // you know when you sit back in your chair and think "damn, I'm really clever"
    let font = asset_server.load("fonts/oldschool_pc_font_pack/Mx437_IBM_VGA_8x16.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 16.0,
        ..default()
    };

    let size = Extent3d {
        width: 640,
        // you may notice that we set the height to 400 here, but later
        // set the height of the cuboid to a 4:3 ratio, i.e. 480.
        // this is because the font we're using, which is an IBM VGA font,
        // was originally stretched slightly in this exact aspect ratio (i.e. it was
        // rendered to a 640x400 pixel grid, but that grid was stretched on the CRT monitor
        // to fill a 640x480 area).
        // See the font website: https://int10h.org/oldschool-pc-fonts/fontlist/font?ibm_vga_8x16
        height: 400,
        ..default()
    };
    // The image object the screen will be rendered to
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);

    // Add to assets, create handles
    let image_handle = images.add(image);
    let first_pass_layer = RenderLayers::layer(1);

    // The stuff to render to the screen
    commands.spawn((
        ShipOS::new(80, 25),
        Text2dBundle {
            text: Text::from_section("", text_style.clone()),
            text_anchor: Anchor::BottomLeft,
            // I solemnly apologise for using magic numbers here, and I promise I will fix it
            transform: Transform::from_xyz(-320., -200., 0.),
            text_2d_bounds: Text2dBounds {
                size: Vec2::new(640., 400.),
            },
            ..default()
        },
        first_pass_layer.clone(),
    ));

    // Light
    commands.spawn((
        PointLightBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
            ..default()
        },
        RenderLayers::layer(0),
    ));

    // Camera that "sees" the text to render
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: -1,
                target: image_handle.clone().into(),
                ..default()
            },
            ..default()
        },
        first_pass_layer,
    ));

    // Cube
    let cube_handle = meshes.add(Cuboid::new(0.24, 0.18, 0.03));
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh: cube_handle,
            material: material_handle,
            transform: Transform::from_xyz(0.0, 1.5, -0.5).with_rotation(Quat::from_euler(
                EulerRot::YXZ,
                PI,
                PI / 10.0,
                0.0,
            )),
            ..default()
        },
        ScreenCuboid,
    ));
}

// Just for a bit of fun
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<ScreenCuboid>>) {
    for mut transform in query.iter_mut() {
        transform.rotation = Quat::from_rotation_y(PI);
        transform.rotate_z(time.elapsed_seconds_wrapped());
        transform.rotate_x(PI / 10.0);
        transform.rotate_z(-time.elapsed_seconds_wrapped());
    }
}

fn draw_screen(mut query: Query<(&mut Text, &ShipOS)>) {
    for (mut text, processor) in query.iter_mut() {
        text.sections[0].value = processor.get_screen().to_string();
    }
}

// TODO: Handle keyboard capturing being passed between computers and the player character
fn _capture_keyboard(mut query: Query<&mut ShipOS>, mut evr_kbd: EventReader<KeyboardInput>) {
    let mut computer = query.single_mut();
    for ev in evr_kbd.read() {
        if ev.state == ButtonState::Released {
            continue;
        }
        computer.handle_keyboard_input(&ev.logical_key);
    }
}
