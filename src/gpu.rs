use crate::renderer::Renderer;
use std::{thread, time};

enum GpuMode {
    /// Horizontal blanking
    HBlank = 0,
    
    /// Vertical blanking
    VBlank = 1,

    /// Accessing Object Attribute Memory during line drawing
    OAMAccess = 2,

    /// Accessing Object Attribute Memory during line drawing
    VRAMAccess = 3,
}

pub struct Gpu {
    renderer: Renderer,
    mode: GpuMode,
    modeclock: usize,
    pub line: u8,
    pub graphics_ram: Vec<u8>,
    scroll_x: u8,
    scroll_y: u8,
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            renderer: Renderer::new(),
            mode: GpuMode::HBlank,
            modeclock: 0,
            line: 0,
            graphics_ram: vec![0; 8192],
            scroll_x: 0,
            scroll_y: 0,
        }
    }

    pub fn step(&mut self, cycle_nb: usize) { 
        self.modeclock += cycle_nb;
        match self.mode {
            GpuMode::OAMAccess => {
                if self.modeclock >= 80 {
                    self.modeclock = 0;
                    self.mode = GpuMode::VRAMAccess;
                }
            }
            GpuMode::VRAMAccess => {
                if self.modeclock >= 172 {
                    self.modeclock = 0;
                    self.mode = GpuMode::HBlank;
                    self.render_line(self.line);
                    // Write a scanlime to the framebuffer
                }
            }
            GpuMode::HBlank => {
                if self.modeclock >= 204 {
                    self.modeclock = 0;
                    self.line += 1;

                    if self.line == 143 {
                        self.mode = GpuMode::VBlank;
                        // Render full buffer
                        // print!("render frame\n");
                        // sleep (temporary hack)
                        // thread::sleep(time::Duration::from_millis(16));
                        self.renderer.render_frame();
                    } else {
                        self.mode = GpuMode::OAMAccess;
                    }
                }
            }
            GpuMode::VBlank => {
                if self.modeclock >= 456 {
                    self.modeclock = 0;
                    self.line += 1;
                    if self.line > 153 {
                        self.mode = GpuMode::OAMAccess;
                        self.line = 0;
                    }
                }
            }
        }
    }

    pub fn set_scroll_y(&mut self, val: u8) {
        self.scroll_y = val;
        self.tile_map();
    }

    pub fn get_scroll_y(&self) -> u8 {
        self.scroll_y
    }

    fn render_line(&mut self, line: u8) {
        let mut texels: Vec<(u8, u8, u8)> = Vec::new();

        let position_y = line.wrapping_add(self.scroll_y) as usize;
        let tile_row = (position_y / 8) * 32;
        for pixel in 0..160u8 {
            let position_x = pixel.wrapping_add(self.scroll_x) as usize;
            let tile_col = position_x / 8;
            let tile_address = 0x1800 + tile_row + tile_col;
            let tile_id = self.graphics_ram[tile_address] as usize;
            let tile_location = tile_id * 16;
            let line = (position_y % 8) * 2;            
            let data = self.graphics_ram[tile_location + line];
            // if data != 0 {
            //     print!("Reading 0x{:02x}\n", data);
            // }
            let color_bit = 7 - (position_x % 8);            
            let val = (data >> color_bit) & 0b1;
            let val = val * 255;
            texels.push((val, val, val));
        }

        self.renderer.render_line(line as u32, texels);
    }

    fn tile_map(&self) {
        print!("\n");
        for i in 0x8010..0x8020{
            print!("{:02X} ", self.graphics_ram[i-0x8000]);
        }
    }
}