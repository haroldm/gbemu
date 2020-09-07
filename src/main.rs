pub mod emulator;
pub mod mmu;

use emulator::Emulator;

fn main() {
    let mut emulator = Emulator::new();
    emulator.run().unwrap();
}
