pub struct Emulator {
    /// Memory
    memory: Mmu,

    /// All SM83 registers
    regs: Registers    
}

/// Reasons why the VM exited
pub enum VmExit {
    /// The VM exited cleanly
    Exit,
}

pub struct Mmu {
    content: Vec<u8>,
}

impl Mmu {
    pub fn new() -> Mmu {
        Mmu {
            content: Vec::new()
        }
    }

    pub fn read(&self, address: u16, len: u16) -> Result<Vec<u8>, VmExit> {
        // TODO check if we should enforce address alignment
        assert!(address % 4 == 0);

        // TODO check if address and length don't overflow

        Ok(self.content
            [(address/4) as usize..(address/4 + len) as usize].to_vec())
    }
}

/// An enum of every accessible registers (including duplicates)
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
} // TODO add flags register ?

pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            memory: Mmu::new(),
            regs: Registers {
                a: 0,
                b: 0,
                c: 0,
                d: 0,
                e: 0,
                f: 0,
                h: 0,
                l: 0,
                sp: 0,
                pc: 0,
            },
        }
    }

    pub fn run(&mut self) -> Result<(), VmExit> {
        loop {
            // Read 3 bytes at memory address PC
            let instr = self.memory.read(self.regs.pc, 3);

            // Decode the instruction
            {

            }
        }
    }

    // TODO check endianness 
    /// Get a register from the emulator
    pub fn reg(&self, register: Register) -> (Option<u8>, Option<u16>) {
        let mut val_u8: Option<u8> = None;
        let mut val_u16: Option<u16> = None;
        match register {
            Register::AF =>
                val_u16 = Some((self.regs.a as u16 >> 8) + self.regs.f as u16),
            Register::BC =>
                val_u16 = Some((self.regs.b as u16 >> 8) + self.regs.c as u16),
            Register::DE =>
                val_u16 = Some((self.regs.d as u16 >> 8) + self.regs.e as u16),
            Register::HL =>
                val_u16 = Some((self.regs.h as u16 >> 8) + self.regs.l as u16),
            Register::A => val_u8 = Some(self.regs.a),
            Register::B => val_u8 = Some(self.regs.b),
            Register::C => val_u8 = Some(self.regs.c),
            Register::D => val_u8 = Some(self.regs.d),
            Register::E => val_u8 = Some(self.regs.e),
            Register::F => val_u8 = Some(self.regs.f),
            Register::H => val_u8 = Some(self.regs.h),
            Register::L => val_u8 = Some(self.regs.l),
            Register::SP => val_u16 = Some(self.regs.sp),
            Register::PC => val_u16 = Some(self.regs.pc),
        }

        // One of the values must be none.
        assert!(val_u8.is_none() || val_u16.is_none());
        (val_u8, val_u16)
    }

    /// Set a register in the emulator
    pub fn set_reg(&mut self, register: Register,
        val: (Option<u8>, Option<u16>)) -> () {
        assert!(val.0.is_none() || val.1.is_none());
        match register {
            Register::AF => {
                assert!(val.0.is_none() && val.1.is_some());
                let val = val.1.unwrap();
                self.regs.a = (val >> 8) as u8;
                self.regs.f = (val & 0b11111111) as u8;
            },
            Register::BC => {
                assert!(val.0.is_none() && val.1.is_some());
                let val = val.1.unwrap();
                self.regs.b = (val >> 8) as u8;
                self.regs.c = (val & 0b11111111) as u8;
            },
            Register::DE => {
                assert!(val.0.is_none() && val.1.is_some());
                let val = val.1.unwrap();
                self.regs.d = (val >> 8) as u8;
                self.regs.e = (val & 0b11111111) as u8;
            },
            Register::HL => {
                assert!(val.0.is_none() && val.1.is_some());
                let val = val.1.unwrap();
                self.regs.h = (val >> 8) as u8;
                self.regs.l = (val & 0b11111111) as u8;
            },
            Register::A => {
                assert!(val.0.is_some() && val.1.is_none());
                self.regs.a = val.0.unwrap();
            },
            Register::B => {
                assert!(val.0.is_some() && val.1.is_none());
                self.regs.b = val.0.unwrap();
            },
            Register::C => {
                assert!(val.0.is_some() && val.1.is_none());
                self.regs.c = val.0.unwrap();
            },
            Register::D => {
                assert!(val.0.is_some() && val.1.is_none());
                self.regs.d = val.0.unwrap();
            },
            Register::E => {
                assert!(val.0.is_some() && val.1.is_none());
                self.regs.e = val.0.unwrap();
            },
            Register::F => {
                assert!(val.0.is_some() && val.1.is_none());
                self.regs.f = val.0.unwrap();
            },
            Register::H => {
                assert!(val.0.is_some() && val.1.is_none());
                self.regs.h = val.0.unwrap();
            },
            Register::L => {
                assert!(val.0.is_some() && val.1.is_none());
                self.regs.l = val.0.unwrap();
            },
            Register::SP => {
                assert!(val.0.is_none() && val.1.is_some());
                self.regs.sp = val.1.unwrap();
            },
            Register::PC => {
                assert!(val.0.is_none() && val.1.is_some());
                self.regs.pc = val.1.unwrap();
            },
        }
    }
}

fn main() {
    let emulator = Emulator::new();
}
