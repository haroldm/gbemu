use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, Condvar};

pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 144;
pub const FRAME_LENGTH: usize = WIDTH as usize * HEIGHT as usize * 4;

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
    /// Channel to send pixel data in
    channel: Option<Sender<Box<[u8; FRAME_LENGTH]>>>,
    pair: Option<Arc<(Mutex<bool>, Condvar)>>,

    frame: [u8; FRAME_LENGTH],
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
            channel: None,
            pair: None,

            frame: [0; WIDTH as usize * HEIGHT as usize * 4],
            mode: GpuMode::HBlank,
            modeclock: 0,
            line: 0,
            graphics_ram: vec![0; 8192],
            scroll_x: 0,
            scroll_y: 0,
        }
    }

    pub fn sync(&mut self, channel: Sender<Box<[u8; FRAME_LENGTH]>>,
        pair: Arc<(Mutex<bool>, Condvar)>) {
        self.pair = Some(pair);
        self.channel = Some(channel);
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
                        // Block thread until previous frame is rendered
                        if let Some(pair) = &self.pair {
                            let (lock, cvar) = &**pair;
                            let mut drawn = lock.lock().unwrap();
                            while !*drawn {
                                drawn = cvar.wait(drawn).unwrap();
                            }
                        }
                        // Render full buffer
                        self.render_frame();
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
    }

    pub fn get_scroll_y(&self) -> u8 {
        self.scroll_y
    }

    fn render_line(&mut self, line: u8) {
        let position_y = line.wrapping_add(self.scroll_y) as usize;
        let tile_row = (position_y / 8) * 32;
        for pixel in 0..160u8 {
            let position_x = pixel.wrapping_add(self.scroll_x) as usize;
            let tile_col = position_x / 8;
            let tile_address = 0x1800 + tile_row + tile_col;
            let tile_id = self.graphics_ram[tile_address] as usize;
            let tile_location = tile_id * 16;
            let line_in_tile = (position_y % 8) * 2;            
            let data = self.graphics_ram[tile_location + line_in_tile];
            let color_bit = 7 - (position_x % 8);            
            let val = (data >> color_bit) & 0b1;
            let val = val * 255;
            let val = [val, val, val, 0xff];

            let offset = (line as usize * WIDTH as usize + pixel as usize) * 4;
            let pixel_in_frame = &mut self.frame[offset..offset+4];
            pixel_in_frame.copy_from_slice(&val);
        }
       
    }

    fn render_frame(&mut self) {
        if let Some(sender) = &self.channel {
            if let Some(pair) = &self.pair {
                let (lock, _) = &**pair;
                let mut drawn = lock.lock().unwrap();
                *drawn = false;
            }
            let _ =  sender.send(Box::new(self.frame));
        }
    }

}