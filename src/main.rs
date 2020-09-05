pub mod emulator;
pub mod mmu;

use emulator::Emulator;

fn main() {
    let mut emulator = Emulator::new();
    //emulator.memory.load_rom("roms/bootrom.gb");
    emulator.run().unwrap();
}
