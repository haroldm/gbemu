// use std::{thread, time};
use crate::mmu::Mmu;

/// Function that makes a closure use same lifetime elision rules as a function
/// cf. https://users.rust-lang.org/t/unhelpful-mismatched-types-error-message/48394/2
fn identity<T, U, F>(f: F) -> F
where
    F: FnMut(&mut T) -> &mut U,
{
    f
}

pub enum CpuFlag {
    C = 0b00010000,
    H = 0b00100000,
    N = 0b01000000,
    Z = 0b10000000,
}
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

impl Registers {
    // TODO check endianness

    /// Get AF register
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | self.f as u16
    }

    /// Get BC register
    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }

    /// Get DE register
    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | self.e as u16
    }

    /// Get HL register
    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }

    /// Set AF register
    pub fn set_af(&mut self, val: u16) {
        self.a = (val >> 8) as u8;
        self.f = (val & 0b11111111) as u8;
    }

    /// Set BC register
    pub fn set_bc(&mut self, val: u16) {
        self.b = (val >> 8) as u8;
        self.c = (val & 0b11111111) as u8;
    }

    /// Set DE register
    pub fn set_de(&mut self, val: u16) {
        self.d = (val >> 8) as u8;
        self.e = (val & 0b11111111) as u8;
    }

    /// Set HL register
    pub fn set_hl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = (val & 0b11111111) as u8;
    }

    pub fn set_flag(&mut self, flag: CpuFlag, val: bool) {
        self.f = match val {
            true => self.f | flag as u8,
            false => self.f & !(flag as u8),
        };
    }

    pub fn flag(&self, flag: CpuFlag) -> bool {
        match self.f & flag as u8 {
            0 => false,
            _ => true,
        }
    }

    pub fn clear_flags(&mut self) {
        self.f &= 0b00001111;
    }
}

pub struct Emulator {
    /// Memory
    pub memory: Mmu,

    /// All SM83 registers
    regs: Registers,
}

/// Reasons why the VM exited
#[derive(Debug)]
pub enum VmExit {
    /// VM exited cleanly
    Exit,

    /// VM exited after a STOP instruction
    Stop,

    /// VM exited after a HALT instruction
    Halt,

    /// VM exited after an out of bounds read
    OobRead,
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
            let instr = self.memory.read_byte(self.regs.pc)?;

            // print!("Executing instruction at 0x{:04x}\n", self.regs.pc);

            // Decode the instruction and return number of bytes read
            let (bytes_read, machine_cycles) = match instr {
                0x00 => (1, 1), // NOP
                0x01 => {
                    // LD BC, d16
                    self.regs.set_bc(self.memory.read_word(self.regs.pc + 1)?);
                    (3, 3)
                }
                0x02 => {
                    // LD (BC), A
                    self.memory.write_byte(self.regs.bc(), self.regs.a)?;
                    (1, 2)
                }
                0x03 => {
                    // INC BC
                    self.regs.set_bc(self.regs.bc().wrapping_add(1));
                    (1, 2)
                }
                0x04 => {
                    // INC B
                    self.regs.b = self.alu_inc8(self.regs.b);
                    (1, 1)
                }
                0x05 => {
                    // DEC B
                    self.regs.b = self.alu_dec8(self.regs.b);
                    (1, 1)
                }
                0x06 => {
                    // LD B, d8
                    self.regs.b = self.memory.read_byte(self.regs.pc + 1)?;
                    (2, 2)
                }
                0x07 => {
                    // RLCA
                    let tmp = self.regs.a;
                    let carry = (0x80 & tmp) == 0x80;
                    self.regs.a = tmp << 1 | if carry { 1 } else { 0 };
                    self.regs.clear_flags();
                    self.regs.set_flag(CpuFlag::C, carry);
                    (1, 1)
                }
                0x08 => {
                    // LD (a16), SP
                    self.regs.sp = self.memory.read_word(self.regs.pc + 1)?;
                    (3, 5)
                }
                0x09 => {
                    // ADD HL, BC
                    self.alu_add_hl(self.regs.bc());
                    (1, 2)
                }
                0x0A => {
                    // LD A, (BC)
                    self.regs.a = self.memory.read_byte(self.regs.bc())?;
                    (1, 2)
                }
                0x0B => {
                    // DEC BC
                    self.regs.set_bc(self.regs.bc().wrapping_sub(1));
                    (1, 2)
                }
                0x0C => {
                    // INC C
                    self.regs.c = self.alu_inc8(self.regs.c);
                    (1, 1)
                }
                0x0D => {
                    // DEC C
                    self.regs.c = self.alu_dec8(self.regs.c);
                    (1, 1)
                }
                0x0E => {
                    // LD C, d8
                    self.regs.c = self.memory.read_byte(self.regs.pc + 1)?;
                    (2, 2)
                }
                0x0F => {
                    // RRCA
                    let tmp = self.regs.a;
                    let carry = (0x01 & tmp) == 0x01;
                    self.regs.a = tmp >> 1 | if carry { 0x80 } else { 0 };
                    self.regs.clear_flags();
                    self.regs.set_flag(CpuFlag::C, carry);
                    (1, 1)
                }
                0x10 => {
                    // STOP
                    return Err(VmExit::Stop);
                }
                0x11 => {
                    // LD DE, d16
                    self.regs.set_de(self.memory.read_word(self.regs.pc + 1)?);
                    (3, 3)
                }
                0x12 => {
                    // LD (DE), A
                    self.memory.write_byte(self.regs.de(), self.regs.a)?;
                    (1, 2)
                }
                0x13 => {
                    // INC DE
                    self.regs.set_de(self.regs.de().wrapping_add(1));
                    (1, 2)
                }
                0x14 => {
                    // INC D
                    self.regs.d = self.alu_inc8(self.regs.d);
                    (1, 1)
                }
                0x15 => {
                    // DEC D
                    self.regs.d = self.alu_dec8(self.regs.d);
                    (1, 1)
                }
                0x16 => {
                    // LD D, d8
                    self.regs.d = self.memory.read_byte(self.regs.pc + 1)?;
                    (2, 2)
                }
                0x17 => {
                    // RLA
                    self.alu_rl(|emu: &mut Emulator| &mut emu.regs.a);
                    self.regs.set_flag(CpuFlag::Z, false);
                    (1, 1)
                }
                0x18 => {
                    // JR r8
                    let tmp = self.memory.read_byte(self.regs.pc + 1)?;
                    self.regs.pc = self.regs.pc.wrapping_add(tmp as i8 as u16);
                    (2, 3)
                }
                0x19 => {
                    // ADD HL, DE
                    self.alu_add_hl(self.regs.de());
                    (1, 2)
                }
                0x1A => {
                    // LD A, (DE)
                    self.regs.a = self.memory.read_byte(self.regs.de())?;
                    (1, 2)
                }
                0x1B => {
                    // DEC DE
                    self.regs.set_de(self.regs.de().wrapping_sub(1));
                    (1, 2)
                }
                0x1C => {
                    // INC E
                    self.regs.e = self.alu_inc8(self.regs.e);
                    (1, 1)
                }
                0x1D => {
                    // DEC E
                    self.regs.e = self.alu_dec8(self.regs.e);
                    (1, 1)
                }
                0x1E => {
                    // LD E, d8
                    self.regs.e = self.memory.read_byte(self.regs.pc + 1)?;
                    (2, 2)
                }
                0x1F => {
                    // RRA
                    let tmp = self.regs.a;
                    let carry = (0x01 & tmp) == 0x01;
                    self.regs.a = tmp >> 1;
                    self.regs.clear_flags();
                    self.regs.set_flag(CpuFlag::C, carry);
                    (1, 1)
                }
                0x20 => {
                    // JR NZ,r8
                    if self.regs.flag(CpuFlag::Z) {
                        (2, 2)
                    } else {
                        let tmp = self.memory.read_byte(self.regs.pc + 1)?;
                        self.regs.pc =
                            self.regs.pc.wrapping_add(tmp as i8 as u16);
                        (2, 3)
                    }
                }
                0x21 => {
                    // LD HL, d16
                    self.regs.set_hl(self.memory.read_word(self.regs.pc + 1)?);
                    (3, 3)
                }
                0x22 => {
                    // LD (HL+), A
                    self.memory.write_byte(self.regs.hl(), self.regs.a)?;
                    self.regs.set_hl(self.regs.hl().wrapping_add(1));
                    (1, 2)
                }
                0x23 => {
                    // INC HL
                    self.regs.set_hl(self.regs.hl().wrapping_add(1));
                    (1, 2)
                }
                0x24 => {
                    // INC H
                    self.regs.h = self.alu_inc8(self.regs.h);
                    (1, 1)
                }
                0x25 => {
                    // DEC H
                    self.regs.h = self.alu_dec8(self.regs.h);
                    (1, 1)
                }
                0x26 => {
                    // LD H, d8
                    self.regs.h = self.memory.read_byte(self.regs.pc + 1)?;
                    (2, 2)
                }
                0x27 => {
                    // DAA
                    panic!("DAA :o");
                    // TODO handle this instruction
                }
                0x28 => {
                    // JR Z,r8
                    if !self.regs.flag(CpuFlag::Z) {
                        (2, 2)
                    } else {
                        let tmp = self.memory.read_byte(self.regs.pc + 1)?;
                        self.regs.pc =
                            self.regs.pc.wrapping_add(tmp as i8 as u16);
                        (2, 3)
                    }
                }
                0x29 => {
                    // ADD HL, HL
                    self.alu_add_hl(self.regs.hl());
                    (1, 2)
                }
                0x2A => {
                    // LD A, (HL+)
                    self.regs.a = self.memory.read_byte(self.regs.hl())?;
                    self.regs.set_hl(self.regs.hl().wrapping_add(1));
                    (1, 2)
                }
                0x2B => {
                    // DEC HL
                    self.regs.set_hl(self.regs.hl().wrapping_sub(1));
                    (1, 2)
                }
                0x2C => {
                    // INC L
                    self.regs.l = self.alu_inc8(self.regs.l);
                    (1, 1)
                }
                0x2D => {
                    // DEC L
                    self.regs.l = self.alu_dec8(self.regs.l);
                    (1, 1)
                }
                0x2E => {
                    // LD L, d8
                    self.regs.l = self.memory.read_byte(self.regs.pc + 1)?;
                    (2, 2)
                }
                0x2F => {
                    // CPL
                    self.regs.a = !self.regs.a;
                    self.regs.set_flag(CpuFlag::N, true);
                    self.regs.set_flag(CpuFlag::H, true);
                    (1, 1)
                }
                0x30 => {
                    // JR NC,r8
                    if self.regs.flag(CpuFlag::C) {
                        (2, 2)
                    } else {
                        let tmp = self.memory.read_byte(self.regs.pc + 1)?;
                        self.regs.pc =
                            self.regs.pc.wrapping_add(tmp as i8 as u16);
                        (2, 3)
                    }
                }
                0x31 => {
                    // LD SP, d16
                    self.regs.sp = self.memory.read_word(self.regs.pc + 1)?;
                    (3, 3)
                }
                0x32 => {
                    // LD (HL-), A
                    self.memory.write_byte(self.regs.hl(), self.regs.a)?;
                    self.regs.set_hl(self.regs.hl().wrapping_sub(1));
                    (1, 2)
                }
                0x33 => {
                    // INC SP
                    self.regs.sp = self.regs.sp.wrapping_add(1);
                    (1, 2)
                }
                0x34 => {
                    // INC (HL)
                    let tmp = self.memory.read_byte(self.regs.hl())?;
                    let tmp = self.alu_inc8(tmp);
                    self.memory.write_byte(self.regs.hl(), tmp)?;
                    (1, 3)
                }
                0x35 => {
                    // DEC (HL)
                    let tmp = self.memory.read_byte(self.regs.hl())?;
                    let tmp = self.alu_dec8(tmp);
                    self.memory.write_byte(self.regs.hl(), tmp)?;
                    (1, 3)
                }
                0x36 => {
                    // LD (HL), d8
                    let tmp = self.memory.read_byte(self.regs.pc + 1)?;
                    self.memory.write_byte(self.regs.hl(), tmp)?;
                    (2, 3)
                }
                0x37 => {
                    // SCF
                    self.regs.set_flag(CpuFlag::N, false);
                    self.regs.set_flag(CpuFlag::H, false);
                    self.regs.set_flag(CpuFlag::C, true);
                    (1, 1)
                }
                0x38 => {
                    // JR C,r8
                    if !self.regs.flag(CpuFlag::C) {
                        (2, 2)
                    } else {
                        let tmp = self.memory.read_byte(self.regs.pc + 1)?;
                        self.regs.pc =
                            self.regs.pc.wrapping_add(tmp as i8 as u16);
                        (2, 3)
                    }
                }
                0x39 => {
                    // ADD HL, SP
                    self.alu_add_hl(self.regs.sp);
                    (1, 2)
                }
                0x3A => {
                    // LD A, (HL-)
                    self.regs.a = self.memory.read_byte(self.regs.hl())?;
                    self.regs.set_hl(self.regs.hl().wrapping_sub(1));
                    (1, 2)
                }
                0x3B => {
                    // DEC SP
                    self.regs.sp = self.regs.sp.wrapping_sub(1);
                    (1, 2)
                }
                0x3C => {
                    // INC A
                    self.regs.a = self.alu_inc8(self.regs.a);
                    (1, 1)
                }
                0x3D => {
                    // DEC A
                    self.regs.a = self.alu_dec8(self.regs.a);
                    (1, 1)
                }
                0x3E => {
                    // LD A, d8
                    self.regs.a = self.memory.read_byte(self.regs.pc + 1)?;
                    (2, 2)
                }
                0x3F => {
                    // CCF
                    self.regs.set_flag(CpuFlag::N, false);
                    self.regs.set_flag(CpuFlag::H, false);
                    self.regs.set_flag(CpuFlag::C, !self.regs.flag(CpuFlag::C));
                    (1, 1)
                }
                0x40..=0x6F | 0x78..=0x7F => {
                    // LD r8, r8
                    // Match on the first three bytes
                    let src = match instr & 0x7 {
                        0x0 => self.regs.b,
                        0x1 => self.regs.c,
                        0x2 => self.regs.d,
                        0x3 => self.regs.e,
                        0x4 => self.regs.h,
                        0x5 => self.regs.l,
                        0x6 => self.memory.read_byte(self.regs.hl())?,
                        0x7 => self.regs.a,
                        _ => unreachable!(),
                    };
                    let dest = match instr & 0b11111000 {
                        0x40 => &mut self.regs.b,
                        0x48 => &mut self.regs.c,
                        0x50 => &mut self.regs.d,
                        0x58 => &mut self.regs.e,
                        0x60 => &mut self.regs.h,
                        0x68 => &mut self.regs.l,
                        0x78 => &mut self.regs.a,
                        _ => unreachable!(),
                    };
                    *dest = src;
                    (1, if instr & 0x7 == 0x6 { 2 } else { 1 })
                }
                0x70..=0x77 => {
                    // LD (HL), r8
                    let src = match instr & 0x7 {
                        0x0 => self.regs.b,
                        0x1 => self.regs.c,
                        0x2 => self.regs.d,
                        0x3 => self.regs.e,
                        0x4 => self.regs.h,
                        0x5 => self.regs.l,
                        0x6 => return Err(VmExit::Halt),
                        0x7 => self.regs.a,
                        _ => unreachable!(),
                    };
                    self.memory.write_byte(self.regs.hl(), src)?;
                    (1, 2)
                }
                0x80..=0xBF => {
                    // Match on the first three bytes
                    let src = match instr & 0x7 {
                        0x0 => self.regs.b,
                        0x1 => self.regs.c,
                        0x2 => self.regs.d,
                        0x3 => self.regs.e,
                        0x4 => self.regs.h,
                        0x5 => self.regs.l,
                        0x6 => self.memory.read_byte(self.regs.hl())?,
                        0x7 => self.regs.a,
                        _ => unreachable!(),
                    };
                    match instr & 0b11111000 {
                        0x80 => self.alu_add(src),
                        0x88 => self.alu_adc(src),
                        0x90 => self.alu_sub(src),
                        0x98 => self.alu_sbc(src),
                        0xA0 => self.alu_and(src),
                        0xA8 => self.alu_xor(src),
                        0xB0 => self.alu_or(src),
                        0xB8 => self.alu_cp(src),
                        _ => unreachable!(),
                    };
                    (1, if instr & 0x7 == 0x6 { 2 } else { 1 })
                }
                0xC0 => {
                    // RET NZ
                    if self.regs.flag(CpuFlag::Z) {
                        (1, 2)
                    } else {
                        self.regs.pc = self.pop16()?;
                        (0, 5)
                    }
                }
                0xC1 => {
                    // POP BC
                    let bc = self.pop16()?;
                    self.regs.set_bc(bc);
                    (1, 3)
                }
                0xC2 => {
                    // JP NZ, a16
                    if self.regs.flag(CpuFlag::Z) {
                        (3, 3)
                    } else {
                        self.regs.pc =
                            self.memory.read_word(self.regs.pc + 1)?;
                        (3, 4)
                    }
                }
                0xC3 => {
                    // JP a16
                    self.regs.pc = self.memory.read_word(self.regs.pc + 1)?;
                    (3, 4)
                }
                0xC4 => {
                    // CALL NZ, a16
                    if self.regs.flag(CpuFlag::Z) {
                        (3, 3)
                    } else {
                        self.push16(self.regs.pc + 2);
                        self.regs.pc = self.memory.read_word(self.regs.pc + 1)?;
                        (0, 6)
                    }
                }
                0xC5 => {
                    // PUSH BC
                    self.push16(self.regs.bc());
                    (1, 4)
                }
                0xC6 => {
                    // ADD A, d8
                    let src = self.memory.read_byte(self.regs.pc + 1)?;
                    self.alu_add(src);
                    (2, 2)
                }
                0xC7 => {
                    // RST 00h
                    self.regs.pc = 0;
                    (1, 4)
                }
                0xC8 => {
                    // RET Z
                    if !self.regs.flag(CpuFlag::Z) {
                        (1, 2)
                    } else {
                        self.regs.pc = self.pop16()?;
                        (0, 5)
                    }
                }
                0xC9 => {
                    // RET
                    self.regs.pc = self.pop16()?;
                    (0, 4)
                }
                0xCA => {
                    // JP Z,a16
                    if !self.regs.flag(CpuFlag::Z) {
                        (3, 3)
                    } else {
                        self.regs.pc =
                            self.memory.read_word(self.regs.pc + 1)?;
                        (3, 4)
                    }
                }
                0xCB => {
                    // PREFIX CB
                    let subinstr = self.memory.read_byte(self.regs.pc + 1)?;

                    let get_src_reg = match subinstr & 0x7 {
                        0x0 => identity(|emu: &mut Emulator| &mut emu.regs.b),
                        0x1 => identity(|emu: &mut Emulator| &mut emu.regs.c),
                        0x2 => identity(|emu: &mut Emulator| &mut emu.regs.d),
                        0x3 => identity(|emu: &mut Emulator| &mut emu.regs.e),
                        0x4 => identity(|emu: &mut Emulator| &mut emu.regs.h),
                        0x5 => identity(|emu: &mut Emulator| &mut emu.regs.l),
                        0x6 => identity(|emu: &mut Emulator| {
                            emu.memory.get_mut_ref_byte(emu.regs.hl()).unwrap()
                        }),
                        0x7 => identity(|emu: &mut Emulator| &mut emu.regs.a),
                        _ => unreachable!(),
                    };

                    match subinstr & 0b11111000 {
                        // 0x00 => self.alu_rlc(src),
                        0x78 => {
                            let tmp = *get_src_reg(self);
                            self.bit(tmp, 7)
                        }
                        0x10 => self.alu_rl(get_src_reg),
                        0x30 => self.alu_swap(get_src_reg),
                        0x38 => self.alu_srl(get_src_reg),

                        /*
                        0x08 => rrc,
                        0x10 => rl,
                        0x18 => rr,
                        0x20 => sla,
                        0x28 => sra,
                        0x40 => bit0,
                        0x48 => bit1,
                        0x50 => bit2,
                        0x58 => bit3,
                        0x60 => bit4,
                        0x68 => bit5,
                        0x70 => bit6,
                        0x78 => bit7,
                        0x80 => res0,
                        0x88 => res1,
                        0x90 => res2,
                        0x98 => res3,
                        0xA0 => res4,
                        0xA8 => res5,
                        0xB0 => res6,
                        0xB8 => res7,
                        0xC0 => set0,
                        0xC8 => set1,
                        0xD0 => set2,
                        0xD8 => set3,
                        0xE0 => set4,
                        0xE8 => set5,
                        0xF0 => set6,
                        0xF8 => set7,
                        */
                        _ => panic!(
                            "Unimplemented for now {:04x} {:02x}",
                            self.regs.pc,
                            self.memory.read_byte(self.regs.pc + 1)?
                        ),
                    }

                    (2, if subinstr & 0x7 == 0x6 { 4 } else { 2 })
                }
                0xCC => {
                    // CALL Z,a16
                    if !self.regs.flag(CpuFlag::Z) {
                        (3, 3)
                    } else {
                        self.push16(self.regs.pc + 2);
                        self.regs.pc = self.memory.read_word(self.regs.pc + 1)?;
                        (0, 6)
                    }
                }
                0xCD => {
                    // CALL a16
                    self.push16(self.regs.pc + 2);
                    self.regs.pc = self.memory.read_word(self.regs.pc + 1)?;
                    (0, 6)
                }
                0xCE => {
                    // ADC A,d8
                    let src = self.memory.read_byte(self.regs.pc + 1)?;
                    self.alu_adc(src);
                    (2, 2)
                }
                0xCF => {
                    // RST 08h
                    self.regs.pc = 0x08;
                    (1, 4)
                }
                0xD0 => {
                    // RET NC
                    if self.regs.flag(CpuFlag::C) {
                        (1, 2)
                    } else {
                        self.regs.pc = self.pop16()?;
                        (0, 5)
                    }
                }
                0xD1 => {
                    // POP DE
                    let de = self.pop16()?;
                    self.regs.set_de(de);
                    (1, 3)
                }
                0xD2 => {
                    // JP NC, a16
                    if self.regs.flag(CpuFlag::C) {
                        (3, 3)
                    } else {
                        self.regs.pc =
                            self.memory.read_word(self.regs.pc + 1)?;
                        (3, 4)
                    }
                }
                0xD4 => {
                    // CALL NC, a16
                    if self.regs.flag(CpuFlag::C) {
                        (3, 3)
                    } else {
                        self.push16(self.regs.pc + 2);
                        self.regs.pc = self.memory.read_word(self.regs.pc + 1)?;
                        (0, 6)
                    }
                }
                0xD5 => {
                    // PUSH DE
                    self.push16(self.regs.de());
                    (1, 4)
                }
                0xD6 => {
                    // SUB d8
                    let src = self.memory.read_byte(self.regs.pc + 1)?;
                    self.alu_sub(src);
                    (2, 2)
                }
                0xD7 => {
                    // RST 10h
                    self.regs.pc = 0x10;
                    (1, 4)
                }
                0xD8 => {
                    // RET C
                    if !self.regs.flag(CpuFlag::C) {
                        (1, 2)
                    } else {
                        self.regs.pc = self.pop16()?;
                        (0, 5)
                    }
                }
                0xD9 => {
                    // RETI
                    self.regs.pc = self.pop16()?;
                    // TODO enable interrupts
                    (0, 4)
                }
                0xDA => {
                    // JP C,a16
                    if !self.regs.flag(CpuFlag::C) {
                        (3, 3)
                    } else {
                        self.regs.pc =
                            self.memory.read_word(self.regs.pc + 1)?;
                        (3, 4)
                    }
                }
                0xDC => {
                    // CALL C,a16
                    if !self.regs.flag(CpuFlag::C) {
                        (3, 3)
                    } else {
                        self.push16(self.regs.pc + 2);
                        self.regs.pc = self.memory.read_word(self.regs.pc + 1)?;
                        (0, 6)
                    }
                }
                0xDE => {
                    // SBC A,d8
                    let src = self.memory.read_byte(self.regs.pc + 1)?;
                    self.alu_sbc(src);
                    (2, 2)
                }
                0xDF => {
                    // RST 18h
                    self.regs.pc = 0x18;
                    (1, 4)
                }
                0xE0 => {
                    // LDH (a8),A
                    let address = self.memory.read_byte(self.regs.pc + 1)?
                        as u16
                        | 0xFF00;
                    self.memory.write_byte(address, self.regs.a)?;
                    (2, 3)
                }
                0xE1 => {
                    // POP HL
                    let hl = self.pop16()?;
                    self.regs.set_hl(hl);
                    (1, 3)
                }
                0xE2 => {
                    // LD (C), A
                    let address = self.regs.c as u16 | 0xFF00;
                    self.memory.write_byte(address, self.regs.a)?;
                    (1, 2)
                }
                0xE5 => {
                    // PUSH HL
                    self.push16(self.regs.hl());
                    (1, 4)
                }
                0xE6 => {
                    // AND d8
                    let src = self.memory.read_byte(self.regs.pc + 1)?;
                    self.alu_and(src);
                    (2, 2)
                }
                0xE7 => {
                    // RST 20h
                    self.regs.pc = 0x20;
                    (1, 4)
                }
                0xE8 => {
                    // ADD SP,r8 add signed
                    // TODO check if add signed changes smth
                    self.regs.set_flag(CpuFlag::N, false);
                    self.regs.set_flag(CpuFlag::Z, false);
                    let val = self.memory.read_byte(self.regs.pc + 1)?;
                    if (val as usize + self.regs.sp as usize) >= 2usize.pow(8) {
                        self.regs.set_flag(CpuFlag::C, true);
                    } else {
                        self.regs.set_flag(CpuFlag::C, false);
                    }
                    if (val as usize + self.regs.sp as usize) >= 2usize.pow(4) {
                        self.regs.set_flag(CpuFlag::H, true);
                    } else {
                        self.regs.set_flag(CpuFlag::H, false);
                    }
                    self.regs.sp = self.regs.sp.wrapping_add(val as u16);
                    (2, 4)
                }
                0xE9 => {
                    // JP (HL)
                    self.regs.pc = self.memory.read_word(self.regs.hl())?;
                    (1, 1)
                }
                0xEA => {
                    // LD (a16), A
                    let address = self.memory.read_word(self.regs.pc + 1)?;
                    self.memory.write_byte(address, self.regs.a)?;
                    (3, 4)
                }
                0xEE => {
                    // XOR d8
                    let src = self.memory.read_byte(self.regs.pc + 1)?;
                    self.alu_xor(src);
                    (2, 2)
                }
                0xEF => {
                    // RST 28h
                    self.regs.pc = 0x28;
                    (1, 4)
                }
                0xF0 => {
                    // LDH A, (a8)
                    let tmp = self.memory.read_byte(self.regs.pc + 1)?;
                    self.regs.a = self.memory.read_byte(tmp as u16 | 0xFF00)?;
                    (2, 3)
                }
                0xF1 => {
                    // POP AF
                    let af = self.pop16()?;
                    self.regs.set_af(af);
                    (1, 3)
                }
                0xF2 => {
                    // LD A,(C)
                    let address = self.regs.c as u16 | 0xFF00;
                    self.regs.a = self.memory.read_byte(address)?;
                    (1, 2)
                }
                0xF3 => {
                    // DI
                    // TODO DI
                    (1, 1)
                }
                0xF5 => {
                    // PUSH AF
                    self.push16(self.regs.af());
                    (1, 4)
                }
                0xF6 => {
                    // OR d8
                    let src = self.memory.read_byte(self.regs.pc + 1)?;
                    self.alu_or(src);
                    (2, 2)
                }
                0xF7 => {
                    // RST 30h
                    self.regs.pc = 0x30;
                    (1, 4)
                }
                0xF8 => {
                    // LD HL, SP+r8
                    self.regs.set_flag(CpuFlag::N, false);
                    self.regs.set_flag(CpuFlag::Z, false);
                    let val = self.memory.read_byte(self.regs.pc + 1)?;
                    if (val as usize + self.regs.sp as usize) >= 2usize.pow(8) {
                        self.regs.set_flag(CpuFlag::C, true);
                    } else {
                        self.regs.set_flag(CpuFlag::C, false);
                    }
                    if (val as usize + self.regs.sp as usize) >= 2usize.pow(4) {
                        self.regs.set_flag(CpuFlag::H, true);
                    } else {
                        self.regs.set_flag(CpuFlag::H, false);
                    }
                    self.regs.set_hl(self.regs.sp.wrapping_add(val as u16));
                    (2, 3)
                }
                0xF9 => {
                    // LD SP, HL
                    self.regs.sp = self.regs.hl();
                    (1, 2)
                }
                0xFA => {
                    // LD A,(a16)
                    let address = self.memory.read_word(self.regs.pc + 1)?;
                    self.regs.a = self.memory.read_byte(address)?;
                    (3, 4)
                }
                0xFB => {
                    // EI
                    // TODO EI
                    (1, 1)
                }
                0xFE => {
                    // CP d8
                    let src = self.memory.read_byte(self.regs.pc + 1)?;
                    self.alu_cp(src);
                    (2, 2)
                }
                0xFF => {
                    // RST 38h
                    self.regs.pc = 0x38;
                    (1, 4)
                }
                _ => unreachable!("Unknown instruction {:02x}", instr),
            };

            self.regs.pc += bytes_read;
            self.memory.gpu.step(machine_cycles * 4);
        }
    }

    fn alu_inc8(&mut self, val: u8) -> u8 {
        let res = val.wrapping_add(1);
        self.regs.set_flag(CpuFlag::N, false);
        if res == 0 {
            self.regs.set_flag(CpuFlag::Z, true);
        } else {
            self.regs.set_flag(CpuFlag::Z, false);
        }
        if res & 0b11111 == 0b10000 {
            self.regs.set_flag(CpuFlag::H, true);
        }
        res
    }

    fn alu_dec8(&mut self, val: u8) -> u8 {
        let res = val.wrapping_sub(1);
        self.regs.set_flag(CpuFlag::N, true);
        if res == 0 {
            self.regs.set_flag(CpuFlag::Z, true);
        } else {
            self.regs.set_flag(CpuFlag::Z, false);
        }
        if res & 0b11111 == 0b01111 {
            self.regs.set_flag(CpuFlag::H, true);
        }
        res
    }

    fn alu_add_hl(&mut self, val: u16) {
        self.regs.set_flag(CpuFlag::N, false);
        if (val as usize + self.regs.hl() as usize) >= 2usize.pow(16) {
            self.regs.set_flag(CpuFlag::C, true);
        } else {
            self.regs.set_flag(CpuFlag::C, false);
        }
        if (val as usize + self.regs.hl() as usize) >= 2usize.pow(12) {
            self.regs.set_flag(CpuFlag::H, true);
        } else {
            self.regs.set_flag(CpuFlag::H, false);
        }
        self.regs.set_hl(val.wrapping_add(self.regs.hl()));
    }

    fn alu_set_zero_flag(&mut self) {
        if self.regs.a == 0 {
            self.regs.set_flag(CpuFlag::Z, true);
        } else {
            self.regs.set_flag(CpuFlag::Z, false);
        }
    }

    fn alu_add(&mut self, val: u8) {
        self.regs.set_flag(CpuFlag::N, false);
        if (val as usize + self.regs.a as usize) >= 2usize.pow(8) {
            self.regs.set_flag(CpuFlag::C, true);
        } else {
            self.regs.set_flag(CpuFlag::C, false);
        }
        if (val as usize + self.regs.a as usize) >= 2usize.pow(4) {
            self.regs.set_flag(CpuFlag::H, true);
        } else {
            self.regs.set_flag(CpuFlag::H, false);
        }
        self.regs.a = val.wrapping_add(self.regs.a);
        self.alu_set_zero_flag();
    }

    fn alu_adc(&mut self, val: u8) {
        let val = if self.regs.flag(CpuFlag::C) {
            val.wrapping_add(1)
        } else {
            val
        };
        self.alu_add(val);
    }

    fn alu_sub(&mut self, val: u8) {
        self.regs.set_flag(CpuFlag::N, true);
        if val > self.regs.a {
            self.regs.set_flag(CpuFlag::C, true);
        } else {
            self.regs.set_flag(CpuFlag::C, false);
        }
        // TODO half-carry is not good here
        if (val as usize + self.regs.a as usize) >= 2usize.pow(4) {
            self.regs.set_flag(CpuFlag::H, true);
        } else {
            self.regs.set_flag(CpuFlag::H, false);
        }
        self.regs.a = self.regs.a.wrapping_sub(val);
        self.alu_set_zero_flag();
    }

    fn alu_sbc(&mut self, val: u8) {
        let val = if self.regs.flag(CpuFlag::C) {
            val.wrapping_add(1)
        } else {
            val
        };
        self.alu_sub(val);
    }

    fn alu_and(&mut self, val: u8) {
        self.regs.set_flag(CpuFlag::N, false);
        self.regs.set_flag(CpuFlag::H, true);
        self.regs.set_flag(CpuFlag::C, false);

        self.regs.a &= val;

        self.alu_set_zero_flag();
    }

    fn alu_xor(&mut self, val: u8) {
        self.regs.set_flag(CpuFlag::N, false);
        self.regs.set_flag(CpuFlag::H, true);
        self.regs.set_flag(CpuFlag::C, false);

        self.regs.a ^= val;

        self.alu_set_zero_flag();
    }

    fn alu_or(&mut self, val: u8) {
        self.regs.set_flag(CpuFlag::N, false);
        self.regs.set_flag(CpuFlag::H, false);
        self.regs.set_flag(CpuFlag::C, false);

        self.regs.a |= val;

        self.alu_set_zero_flag();
    }

    fn alu_cp(&mut self, val: u8) {
        self.regs.set_flag(CpuFlag::N, true);
        if val > self.regs.a {
            self.regs.set_flag(CpuFlag::C, true);
        } else {
            self.regs.set_flag(CpuFlag::C, false);
        }
        // TODO half-carry is not good here
        if (val as usize + self.regs.a as usize) >= 2usize.pow(4) {
            self.regs.set_flag(CpuFlag::H, true);
        } else {
            self.regs.set_flag(CpuFlag::H, false);
        }
        let res = self.regs.a.wrapping_sub(val);
        if res == 0 {
            self.regs.set_flag(CpuFlag::Z, true);
        } else {
            self.regs.set_flag(CpuFlag::Z, false);
        }
    }

    fn alu_rl<'a, F: FnMut(&mut Emulator) -> &mut u8>(
        &'a mut self,
        mut get_reg: F,
    ) {
        let zero_flag: bool;
        let old_carry = if self.regs.flag(CpuFlag::C) { 1 } else { 0 };
        let carry: bool;
        {
            let val = get_reg(self);
            carry = (0x80 & *val) == 0x80;
            *val = (*val << 1) | old_carry;
            zero_flag = if *val == 0 { true } else { false };
        }
        self.regs.clear_flags();
        self.regs.set_flag(CpuFlag::C, carry);
        self.regs.set_flag(CpuFlag::Z, zero_flag);
    }

    fn alu_swap<'a, F: FnMut(&mut Emulator) -> &mut u8>(
        &'a mut self,
        mut get_reg: F,
    ) {
        let zero_flag: bool;
        let val = get_reg(self);
        {
            let lower = *val & 0x0f;
            *val = (*val >> 4) | (lower << 4);
            zero_flag = if *val == 0 { true } else { false };
        }
        self.regs.clear_flags();
        self.regs.set_flag(CpuFlag::Z, zero_flag);
    }

    fn alu_srl<'a, F: FnMut(&mut Emulator) -> &mut u8>(
        &'a mut self,
        mut get_reg: F,
    ) {
        let zero_flag: bool;
        let carry: bool;
        {
            let val = get_reg(self);
            carry = (0x01 & *val) == 0x01;
            *val = *val >> 1;
            zero_flag = if *val == 0 { true } else { false };
        }
        self.regs.clear_flags();
        self.regs.set_flag(CpuFlag::C, carry);
        self.regs.set_flag(CpuFlag::Z, zero_flag);
    }

    fn pop16(&mut self) -> Result<u16, VmExit> {
        let res = self.memory.read_word(self.regs.sp)?;
        // self.regs.sp = self.regs.sp.wrapping_add(2);
        self.regs.sp += 2;
        Ok(res)
    }

    fn push16(&mut self, val: u16) {
        self.regs.sp -= 2;
        self.memory.write_word(self.regs.sp, val).unwrap();
    }

    fn bit(&mut self, val: u8, n: u8) {
        assert!(n < 8);

        let tested_bit = 0b1 << n;
        if val & tested_bit == 0 {
            self.regs.set_flag(CpuFlag::Z, true);
        } else {
            self.regs.set_flag(CpuFlag::Z, false);
        }
    }
}
