use std::io;
use std::io::Write;

use mos6502::memory::Bus;
use mos6502::memory::Memory;
use mos6502::instruction::Nmos6502;
use mos6502::cpu;

fn main() {
    let program = [
        0xa9, 0x48,         // LDA #$48 (ascii 'H')
        0x8d, 0x00, 0x02,   // STA $0200
        0xa9, 0x65,         // LDA #$65 (ascii 'e')
        0x8d, 0x01, 0x02,   // STA $0201
        0xa9, 0x6c,         // LDA #$6c (ascii 'l')
        0x8d, 0x02, 0x02,   // STA $0202
        0xa9, 0x6c,         // LDA #$6c (ascii 'l')
        0x8d, 0x03, 0x02,   // STA $0203
        0xa9, 0x6f,         // LDA #$6f (ascii 'o')
        0x8d, 0x04, 0x02,   // STA $0204
        0xff,               // end?
    ];

    let mut cpu = cpu::CPU::new(Memory::new(), Nmos6502);

    cpu.memory.set_bytes(0x10, &program);
    cpu.registers.program_counter = 0x10;

    cpu.run();

    let base: u16 = 0x0200;
    for offset in 0u16..5u16 {
        let loc = base + offset;
        let byte = cpu.memory.get_byte(loc);
        print!("{}", byte as char);
    }
    let _ = io::stdout().flush();
}
