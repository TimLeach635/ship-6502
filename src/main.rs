use std::fs::read;

use buffer_graphics_lib::prelude::NewTextPos;
use buffer_graphics_lib::prelude::TextPos;
use buffer_graphics_lib::prelude::LIGHT_GRAY;
use buffer_graphics_lib::text::PixelFont;
use buffer_graphics_lib::text::Text;
use buffer_graphics_lib::Graphics;
use mos6502::memory::Bus;
use mos6502::memory::Memory;
use mos6502::instruction::Nmos6502;
use mos6502::cpu;
use pixels::Pixels;
use pixels::SurfaceTexture;
use tao::dpi::LogicalSize;
use tao::event::Event;
use tao::event::KeyEvent;
use tao::event::WindowEvent;
use tao::event_loop::ControlFlow;
use tao::event_loop::EventLoop;
use tao::keyboard::KeyCode;
use tao::window::WindowBuilder;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() {
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

    let mut cpu = cpu::CPU::new(Memory::new(), Nmos6502);

    cpu.memory.set_bytes(0x0000, &zp);
    cpu.memory.set_bytes(0x0100, &ram);
    cpu.memory.set_bytes(0x8000, &rom);
    cpu.registers.program_counter = 0x8000;

    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("ship_6502")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture).unwrap()
    };

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                // Close window
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: KeyCode::Escape,
                            ..
                        },
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }

                WindowEvent::Resized(size) => {
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        eprintln!("Error resizing window: {}", err);
                        *control_flow = ControlFlow::Exit;
                    }
                }

                _ => {}
            },

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            Event::RedrawRequested(_) => {
                let mut buffer = pixels.frame_mut();
                let mut graphics = Graphics::new(&mut buffer, SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize).unwrap();

                // get line
                let mut line = String::new();
                for offset in 0..100 {
                    let byte = cpu.memory.get_byte(0x0200 + offset);
                    match byte {
                        0x00 => line.push(0x20 as char),  // render byte 0x00 as a blank space (0x20). A hack, for now
                        byte => line.push(byte as char),
                    }
                }

                let text = Text::new(&line, TextPos::cr((0, 0)), (LIGHT_GRAY, PixelFont::Standard8x10));
                graphics.draw(&text);

                pixels.render().unwrap();
            }

            _ => {}
        }

        cpu.single_step();
    });
}
