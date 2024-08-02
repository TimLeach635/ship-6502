use std::fs::read;

use minifb::Window;
use minifb::WindowOptions;
use mos6502::memory::Bus;
use mos6502::memory::Memory;
use mos6502::instruction::Nmos6502;
use mos6502::cpu;

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

    let mut window = Window::new(
        "ship_6502",
        32,
        32,
        WindowOptions { scale: minifb::Scale::X8, ..Default::default() }
    ).expect("Error opening window");

    while window.is_open() {
        let mut buffer: [u32; 32 * 32] = [0; 32 * 32];
        for offset in 0usize..(32 * 32) {
            let byte = cpu.memory.get_byte(0x0200 + (offset as u16));
            match byte {
                0x00 => buffer[offset] = 0x00000000,
                _ => buffer[offset] = 0x00ffffff,
            }
        }
        window.update_with_buffer(&buffer, 32, 32).expect("Error drawing buffer");

        cpu.single_step();
    }
}
