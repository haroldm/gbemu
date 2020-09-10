pub mod emulator;
pub mod mmu;
pub mod gpu;
pub mod renderer;

use emulator::Emulator;

fn main() {
    let mut emulator = Emulator::new();
    emulator.run().unwrap();
}
