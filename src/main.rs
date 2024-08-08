use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::render::render_resource::TextureDescriptor;
use bevy::render::render_resource::TextureDimension;
use bevy::render::render_resource::TextureFormat;
use bevy::render::render_resource::TextureUsages;
use bevy::render::view::RenderLayers;
use bevy::text::Text;

#[derive(Component)]
struct Screen;

#[derive(Component)]
struct ScreenCuboid;

fn setup_time(mut time: ResMut<Time>) {
    time.set_wrap_period(Duration::from_secs_f32(2.0 * PI));
}

fn setup_screen(
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
        // set the height of the cuboid to 4.8, i.e. 480.
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
        Screen,
        Text2dBundle {
            text: Text::from_section("A quick brown fox jumps over the lazy dog.
0123456789 ¿?¡!`'\"., <>()[]{} &@%*^#$\\/

* Wieniläinen sioux'ta puhuva ökyzombie diggaa Åsan roquefort-tacoja.
* Ça me fait peur de fêter noël là, sur cette île bizarroïde où une mère et sa
môme essaient de me tuer avec un gâteau à la cigüe brûlé.
* Zwölf Boxkämpfer jagten Eva quer über den Sylter Deich.
* El pingüino Wenceslao hizo kilómetros bajo exhaustiva lluvia y frío, añoraba
a su querido cachorro.

┌─┬─┐ ╔═╦═╗ ╒═╤═╕ ╓─╥─╖
│ │ │ ║ ║ ║ │ │ │ ║ ║ ║
├─┼─┤ ╠═╬═╣ ╞═╪═╡ ╟─╫─╢
└─┴─┘ ╚═╩═╝ ╘═╧═╛ ╙─╨─╜

░░░░░ ▐▀█▀▌ .·∙•○°○•∙·.
▒▒▒▒▒ ▐ █ ▌ ☺☻ ♥♦♣♠ ♪♫☼
▓▓▓▓▓ ▐▀█▀▌  $ ¢ £ ¥ ₧
█████ ▐▄█▄▌ ◄►▲▼ ←→↑↓↕↨

⌠
│dx ≡ Σ √x²ⁿ·δx
⌡", text_style.clone()),
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
    let cube_handle = meshes.add(Cuboid::new(6.4, 4.8, 0.5));
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
            transform: Transform::from_xyz(0.0, 0.0, 1.5)
                .with_rotation(Quat::from_euler(EulerRot::YXZ, PI, PI / 10.0, 0.0)),
            ..default()
        },
        ScreenCuboid,
    ));

    // Main pass camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn rotator_system(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<ScreenCuboid>>,
) {
    for mut transform in query.iter_mut() {
        transform.rotation = Quat::from_rotation_y(PI);
        transform.rotate_z(time.elapsed_seconds_wrapped());
        transform.rotate_x(PI / 10.0);
        transform.rotate_z(-time.elapsed_seconds_wrapped());
    }
}

fn draw_screen(
    mut query: Query<&mut Text, With<Screen>>,
) {
    // // Put it in the rendered text
    // for mut text in query.iter_mut() {
    //     text.sections[0].value = line.clone();
    // }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_time, setup_screen))
        .add_systems(Update, (draw_screen, rotator_system))
        .run();
}
