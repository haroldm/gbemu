use crate::emulator::VmExit;
use crate::gpu::Gpu;

pub struct Mmu {
    rom: Vec<u8>,
    bootrom: Vec<u8>,
    bootrom_lock: bool,
    ram: Vec<u8>,
    zero_page_ram: Vec<u8>,
    pub gpu: Gpu,
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
            gpu: Gpu::new(),
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        self.rom = std::fs::read(path).ok().unwrap();
    }

    pub fn read_byte(&mut self, address: u16) -> Result<u8, VmExit> {
        let address = address as usize;
        match address {
            0x0000..=0x7FFF => {
                if self.bootrom_lock == true && address <= 0xFF {
                    return Ok(self.bootrom[address]);
                }
                Ok(self.rom[address])
            }
            0x8000..=0x9FFF => self.gpu.read_byte(address),
            0xC000..=0xDFFF => Ok(self.ram[address - 0xC000]),
            0xE000..=0xFDFF => Ok(self.ram[address - 0xE000]),
            0xFE00..=0xFE9F => self.gpu.read_byte(address),
            0xFF00..=0xFF7F => self.handle_io_read(address),
            0xFF80..=0xFFFF => Ok(self.zero_page_ram[address - 0xFF80]),
            _ => Err(VmExit::Exit),
        }
    }

    pub fn read_word(&mut self, address: u16) -> Result<u16, VmExit> {
        Ok(self.read_byte(address)? as u16
            | (self.read_byte(address + 1)? as u16) << 8)
    }

    pub fn write_byte(&mut self, address: u16, val: u8) -> Result<(), VmExit> {
        let address = address as usize;
        match address {
            // 0x0000..=0x3FFF => Ok(()),
            0x8000..=0x9FFF => self.gpu.write_byte(address, val),
            0xC000..=0xDFFF => {
                self.ram[address - 0xC000] = val;
                Ok(())
            }
            0xFF00..=0xFF7F => self.handle_io_write(address, val),

            0xFF80..=0xFFFF => {
                self.zero_page_ram[address - 0xFF80] = val;
                Ok(())
            }
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

    pub fn get_mut_ref_byte(
        &mut self,
        address: u16,
    ) -> Result<&mut u8, VmExit> {
        Ok(&mut self.rom[address as usize])
    }

    fn handle_io_write(
        &mut self,
        address: usize,
        val: u8,
    ) -> Result<(), VmExit> {
        match address {
            0xFF11 => {
                // NR11 - Channel 1 Sound length/Wave pattern duty (R/W)
                Ok(())
            }
            0xFF12 => {
                // NR12 - Channel 1 Volume Envelope (R/W)
                Ok(())
            }
            0xFF13 => {
                // NR13 - Channel 1 Frequency lo (Write Only)
                Ok(())
            }
            0xFF14 => {
                // NR14 - Channel 1 Frequency hi (R/W)
                Ok(())
            }
            0xFF24 => {
                // NR50 - Channel control / ON-OFF / Volume (R/W)
                Ok(())
            }
            0xFF25 => {
                // NR51 - Selection of Sound output terminal (R/W)
                Ok(())
            }
            0xFF26 => {
                // NR52 Sound on/off
                Ok(())
            }
            0xFF40..=0xFF4F => self.gpu.write_byte(address, val),
            0xFF50 => {
                // Boot ROM lock register
                if val & 0x01 == 0x01
                    && self.read_byte(address as u16)? & 0x01 == 0
                {
                    self.bootrom_lock = false;
                }
                Ok(())
            }
            _ => {
                panic!("Trying to write 0x{:02x} to I/O 0x{:04x}", val, address)
            }
        }
    }

    fn handle_io_read(&mut self, address: usize) -> Result<u8, VmExit> {
        match address {
            0xFF40..=0xFF4F => self.gpu.read_byte(address),
            0xFF50 => {
                // Boot ROM lock register
                Ok(if self.bootrom_lock { 0 } else { 1 })
            }
            0xFF68..=0xFF6B => self.gpu.read_byte(address),
            _ => panic!("Trying to read at I/O 0x{:04x}", address),
        }
    }
}
