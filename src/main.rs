pub mod emulator;
pub mod gpu;
pub mod mmu;

use emulator::Emulator;
use gpu::{FRAME_LENGTH, HEIGHT, WIDTH};

use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;

use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const GRAPHICS_OUTPUT: bool = false;

fn main() {
    let mut emulator = Emulator::new();
    emulator.memory.load_rom("roms/tetris.gb");

    if GRAPHICS_OUTPUT {
        // Start the emulator and sync the GPU
        let (tx, rx) = mpsc::channel();
        let pair = Arc::new((Mutex::new(true), Condvar::new()));
        let pair2 = pair.clone();
        let emulator_thread = thread::spawn(move || {
            emulator.memory.gpu.sync(tx, pair2);
            emulator.run().unwrap();
        });

        let event_loop = EventLoop::new();
        let mut input = WinitInputHelper::new();
        let window = {
            let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
            WindowBuilder::new()
                .with_title("GBEMU")
                .with_inner_size(size)
                .with_min_inner_size(size)
                .build(&event_loop)
                .unwrap()
        };

        let mut pixels = {
            let window_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(
                window_size.width,
                window_size.height,
                &window,
            );
            Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
        };

        let mut frame = [0; FRAME_LENGTH];

        event_loop.run(move |event, _, control_flow| {
            emulator_thread.thread().id();

            // Draw the current frame
            if let Event::RedrawRequested(_) = event {
                match rx.recv() {
                    Ok(buffer) => {
                        frame = *buffer;
                        let (lock, cvar) = &*pair;
                        let mut drawn = lock.lock().unwrap();
                        *drawn = true;
                        cvar.notify_one();
                    }
                    Err(_) => (),
                }

                pixels.get_frame().copy_from_slice(&frame);

                if pixels
                    .render()
                    .map_err(|e| error!("pixels.render() failed: {}", e))
                    .is_err()
                {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            // Handle input events
            if input.update(event) {
                // Close events
                if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                // Resize the window
                if let Some(size) = input.window_resized() {
                    pixels.resize(size.width, size.height);
                }

                // Request a redraw
                window.request_redraw();
            }
        });
    } else {
        emulator.run().unwrap();
    }
}
