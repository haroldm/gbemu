use crate::emulator::VmExit;

use std::convert::TryInto;

pub struct Mmu {
    rom: Vec<u8>,
    bootrom: Vec <u8>,
    bootrom_lock: bool,
    ram: Vec<u8>,
}

impl Mmu {
    pub fn new() -> Mmu {
        let bootrom = std::fs::read("roms/bootrom.gb").ok().unwrap();
        Mmu {
            bootrom: bootrom,
            bootrom_lock: true,
            rom: vec![0; 32768],
            ram: vec![0; 16384],
        }
    }


    pub fn load_rom(&mut self, rom_path: &str) {

    }

    // pub fn read(&self, address: u16, len: u16) -> Result<Vec<u8>, VmExit> {
    //     // TODO check if address and length don't overflow
    //     Ok(self.rom
    //         [address as usize..(address + len) as usize].to_vec()
    //     )
    // }

    pub fn read_byte(&self, address: u16) -> Result<u8, VmExit> {
        match address {
            0x0000..=0x7FFF => {
                if self.bootrom_lock == true && address < 0xFF {
                    Ok(self.bootrom[address as usize])
                } else {
                    Ok(0)
                    //Err(VmExit::OobRead)
                }
            }
            0xC000..=0xDFFF => Ok(self.ram[address as usize]),
            _ => Ok(0),
            //_ => Err(VmExit::OobRead),
        }
    }

    pub fn read_word(&self, address: u16) -> Result<u16, VmExit> {
        Ok(self.read_byte(address)? as u16 |
            (self.read_byte(address + 1)? as u16) << 8
        )
    }

    pub fn write_byte(&mut self, address: u16, val: u8) -> Result<(), VmExit> {
        match address {
            0xC000..=0xDFFF => {
                self.ram[address as usize] = val;
                Ok(())
            },
            0xFF50 => { // Boot ROM lock register
                if val & 0x01== 0x01 && self.read_byte(address)? & 0x01 == 0 {
                    self.bootrom_lock = false;
                    panic!("Switching out of bootrom");
                }
                Ok(())
            }
            _ => Ok(()),
            //_ => Err(VmExit::OobRead),
        }
    }

    pub fn write_word(&mut self, address: u16, val: u16) -> Result<(), VmExit> {
        self.write_byte(address, (val & 0xFF) as u8)?;
        self.write_byte(address + 1, (val >> 8) as u8)?;
        Ok(())
    }

    pub fn get_mut_ref_byte(&mut self, address: u16) -> Result<&mut u8, VmExit> {
        Ok(&mut self.rom[address as usize])
    }
}
