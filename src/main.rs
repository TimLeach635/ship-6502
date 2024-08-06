use std::fs::read;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use buffer_graphics_lib::prelude::NewTextPos;
use buffer_graphics_lib::prelude::TextPos;
use buffer_graphics_lib::prelude::LIGHT_GRAY;
use buffer_graphics_lib::text::PixelFont;
use buffer_graphics_lib::text::Text;
use buffer_graphics_lib::Graphics;
use image::DynamicImage;
use image::RgbaImage;
use mos6502::cpu;
use mos6502::cpu::CPU;
use mos6502::instruction::Nmos6502;
use mos6502::memory::Bus;
use mos6502::memory::Memory;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

#[derive(Resource)]
struct Cpu(CPU<Memory, Nmos6502>);

#[derive(Component)]
struct Screen;

fn setup_cpu(mut commands: Commands) {
    // Load 6502 program
    let zp = match read("asm/zp.bin") {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error reading zp.bin: {}", err);
            return;
        }
    };
    let ram = match read("asm/ram.bin") {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error reading ram.bin: {}", err);
            return;
        }
    };
    let rom = match read("asm/rom.bin") {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error reading rom.bin: {}", err);
            return;
        }
    };

    // Initialise 6502, load program into its memory
    let mut cpu = cpu::CPU::new(Memory::new(), Nmos6502);
    cpu.memory.set_bytes(0x0000, &zp);
    cpu.memory.set_bytes(0x0100, &ram);
    cpu.memory.set_bytes(0x8000, &rom);
    cpu.registers.program_counter = 0x8000;

    // Initialise Bevy global CPU resource
    commands.insert_resource(Cpu(cpu));
}

fn setup_screen(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((Screen, SpriteBundle::default()));
}

fn draw_screen(
    mut cpu: ResMut<Cpu>,
    mut screens: Query<&mut Handle<Image>, With<Screen>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Draw text to buffer
    let mut buffer: [u8; (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize] =
        [0; (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize];
    let mut graphics =
        Graphics::new(&mut buffer, SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize).unwrap();
    let mut line = String::new();
    for offset in 0..100 {
        let byte = cpu.0.memory.get_byte(0x0200 + offset);
        match byte {
            0x00 => line.push(0x20 as char), // render byte 0x00 as a blank space (0x20). A hack, for now
            byte => line.push(byte as char),
        }
    }
    let text = Text::new(
        &line,
        TextPos::cr((0, 0)),
        (LIGHT_GRAY, PixelFont::Standard8x10),
    );
    graphics.draw(&text);

    // Load buffer into Bevy as an Image
    let screen_image_buffer =
        RgbaImage::from_raw(SCREEN_WIDTH, SCREEN_HEIGHT, buffer.to_vec()).unwrap();
    let screen_image = Image::from_dynamic(
        DynamicImage::ImageRgba8(screen_image_buffer),
        false,
        RenderAssetUsages::RENDER_WORLD,
    );

    let screen_image_handle = images.add(screen_image);

    // Draw image to all screens
    for mut handle in screens.iter_mut() {
        *handle = screen_image_handle.clone_weak();
    }
}

fn step_cpu(mut cpu: ResMut<Cpu>) {
    cpu.0.single_step();
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_cpu, setup_screen))
        .add_systems(Update, (draw_screen, step_cpu))
        .run();
}
