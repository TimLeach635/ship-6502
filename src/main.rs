use std::fs::read;

use bevy::prelude::*;
use bevy::text::Text;
use mos6502::cpu;
use mos6502::cpu::CPU;
use mos6502::instruction::Nmos6502;
use mos6502::memory::Bus;
use mos6502::memory::Memory;

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

fn setup_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/oldschool_pc_font_pack/Mx437_IBM_VGA_8x16.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 16.0,
        ..default()
    };

    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        Screen,
        Text2dBundle {
            text: Text::from_section("", text_style.clone()),
            ..default()
        },
    ));
}

fn draw_screen(
    mut cpu: ResMut<Cpu>,
    mut query: Query<&mut Text, With<Screen>>,
) {
    // Extract text from CPU's memory
    let mut line = String::new();
    for offset in 0..100 {
        let byte = cpu.0.memory.get_byte(0x0200 + offset);
        match byte {
            0x00 => line.push(0x20 as char), // render byte 0x00 as a blank space (0x20). A hack, for now
            byte => line.push(byte as char),
        }
    }

    // Put it in the rendered text
    for mut text in query.iter_mut() {
        text.sections[0].value = line.clone();
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
