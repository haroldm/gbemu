use crate::emulator::VmExit;

static NINTENDO_LOGO: [u8; 76] = [
    // Logo
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83,
    0x00, 0x0C, 0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
    0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63,
    0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
    // Game title, checksum to 0xe7
    0xe7, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00
];

pub struct Mmu {
    rom: Vec<u8>,
    bootrom: Vec <u8>,
    bootrom_lock: bool,
    ram: Vec<u8>,
    zero_page_ram: Vec<u8>,
    graphics_ram: Vec<u8>,
    scroll_y: u8,
}

impl Mmu {
    pub fn new() -> Mmu {
        let bootrom = std::fs::read("roms/bootrom.gb").ok().unwrap();
        Mmu {
            bootrom: bootrom,
            bootrom_lock: true,
            rom: vec![0; 32768],
            ram: vec![0; 8192],
            zero_page_ram: vec![0; 128],
            graphics_ram: vec![0; 8192],
            scroll_y: 0,
        }
    }

    pub fn read_byte(&mut self, address: u16) -> Result<u8, VmExit> {
        let address = address as usize;
        match address {
            0x0000..=0x7FFF => {
                if self.bootrom_lock == true && address <= 0xFF {
                    return Ok(self.bootrom[address])
                }
                // print!("Reading 0x{:04x}\n", address);
                if address >= 0x104 && address <= 0x14d {
                    return Ok(NINTENDO_LOGO[address - 0x104])
                }
                Ok(self.rom[address])
            }
            0x8000..=0x9FFF => Ok(self.graphics_ram[address - 0x8000]),
            0xC000..=0xDFFF => Ok(self.ram[address - 0xC000]),
            0xE000..=0xFDFF => Ok(self.ram[address - 0xE000]),
            0xFE00..=0xFE9F => panic!("sprite data"),
            0xFF00..=0xFF7F => self.handle_io_read(address),
            0xFF80..=0xFFFF => Ok(self.zero_page_ram[address - 0xFF80]),
            _ => Err(VmExit::Exit),
        }
    }

    pub fn read_word(&mut self, address: u16) -> Result<u16, VmExit> {
        Ok(self.read_byte(address)? as u16 |
            (self.read_byte(address + 1)? as u16) << 8
        )
    }

    pub fn write_byte(&mut self, address: u16, val: u8) -> Result<(), VmExit> {
        let address = address as usize;
        match address {
            // 0x0000..=0x3FFF => Ok(()),
            0x8000..=0x9FFF => {
                self.graphics_ram[address - 0x8000] = val;
                Ok(())
            },
            0xC000..=0xDFFF => {
                self.ram[address - 0xC000] = val;
                Ok(())
            },
            0xFF00..=0xFF7F => {
                self.handle_io_write(address, val)
            }
            
            0xFF80..=0xFFFF => {
                self.zero_page_ram[address - 0xFF80] = val;
                Ok(())
            },
            _ => panic!(
                "Trying to write byte 0x{:02x} at address 0x{:04x}",
                val, address
            ),
        }
    }

    pub fn write_word(&mut self, address: u16, val: u16) -> Result<(), VmExit> {
        // print!("Writing {:04x} at {:04x}\n", val, address);
        self.write_byte(address, (val & 0xFF) as u8)?;
        self.write_byte(address + 1, (val >> 8) as u8)?;
        Ok(())
    }

    pub fn get_mut_ref_byte(&mut self, address: u16) -> Result<&mut u8, VmExit> {
        Ok(&mut self.rom[address as usize])
    }

    fn handle_io_write(&mut self, address: usize, val: u8) -> Result<(), VmExit> {
        match address {
            0xFF11 => { // NR11 - Channel 1 Sound length/Wave pattern duty (R/W)
                Ok(())
            }
            0xFF13 => { // NR13 - Channel 1 Frequency lo (Write Only)
                Ok(())
            }
            0xFF25 => { // NR51 - Selection of Sound output terminal (R/W)
                Ok(())
            }
            0xFF26 => { // NR52 Sound on/off
                 Ok(())
            }
            0xFF40 => { // LCDC - LCD Control (R/W)
                Ok(())
            }
            0xFF42 => { // SCY - Scroll Y (R/W)
                self.scroll_y = val;
                Ok(())
            }
            0xFF47 => { // BGP - BG Palette Data (R/W) - Non CGB Mode Only
                Ok(())
            }
            0xFF50 => { // Boot ROM lock register
                if val & 0x01 == 0x01
                    && self.read_byte(address as u16)? & 0x01 == 0 {
                    self.bootrom_lock = false;
                    panic!("Switching out of bootrom");
                }
                Ok(())
            }
            _ => panic!("Trying to write 0x{:02x} to I/O 0x{:04x}", val, address)
        }
    }

    fn handle_io_read(&mut self, address: usize) -> Result<u8, VmExit> {
        match address {
            0xFF42 => { // SCY - Scroll Y (R/W)
                Ok(self.scroll_y)
            }
            0xFF44 => { // LY - LCDC Y-Coordinate (R)
                Ok(144)
            }
            0xFF50 => { // Boot ROM lock register
                Ok(if self.bootrom_lock { 0 } else { 1 })
            }
            _ => panic!("Trying to read at I/O 0x{:04x}", address)
        }
    }

}
