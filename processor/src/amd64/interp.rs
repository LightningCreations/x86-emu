use crate::ProcessorImplementation;
use bitflags::bitflags;
use file_loader::MemoryMap;

use bytemuck::Zeroable;
use file_loader::Registers;

#[derive(PartialEq)]
pub enum OperandSize {
    R8,
    R16,
    R32,
    R64,
}

pub struct Amd64Interp {
    regs: Registers,
}

impl Amd64Interp {
    pub fn new() -> Amd64Interp {
        Amd64Interp {
            regs: Registers::zeroed(),
        }
    }

    pub fn modrm<T>(&mut self, map: &mut dyn MemoryMap, size: OperandSize, function: T)
    where
        T: Fn(&mut u64, u64, OperandSize),
    {
        let modrm_byte = map.read_u8(self.regs.rip);
        self.regs.rip += 1;
        let mut src = self.regs.gprs[((modrm_byte >> 3) & 0x07) as usize];
        let dst = match modrm_byte & 0xC0 {
            0xC0 => {
                let result = &mut self.regs.gprs[(modrm_byte & 0x7) as usize];
                if size == OperandSize::R32 {
                    *result &= 0x00000000FFFFFFFF; // Zero out higher half
                }
                result
            }
            _ => panic!("Unrecognized ModR/M dst in {:#04X}", modrm_byte),
        };

        if size == OperandSize::R32 {
            src &= 0x00000000FFFFFFFF;
        }
        function(dst, src, size);
    }
}

impl Default for Amd64Interp {
    fn default() -> Self {
        Self::new()
    }
}

bitflags! {
    struct Prefixes: u64 {
        const NONE = 0b00000000;
        const REP = 0b00000001;
        const OPSIZE = 0b00000010;
        const REX = 0b00000100;
        const REX_B = 0b00001000;
        const REX_X = 0b00010000;
        const REX_R = 0b00100000;
        const REX_W = 0b01000000;
    }
}

impl ProcessorImplementation for Amd64Interp {
    fn init(&mut self, map: &mut dyn MemoryMap) {
        self.regs.rip = map.entry_point();
    }
    fn running(&self) -> bool {
        true
    }
    fn tick(&mut self, map: &mut dyn MemoryMap) {
        let mut prefixes = Prefixes::NONE;
        let mut done = false;
        while !done {
            let instr = map.read_u8(self.regs.rip);
            self.regs.rip += 1;
            match instr {
                0x0F => {
                    let instr2 = map.read_u8(self.regs.rip);
                    self.regs.rip += 1;
                    match instr2 {
                        0x1E => {
                            let _ = map.read_u8(self.regs.rip); // ModR/M
                            self.regs.rip += 1;
                            done = true // It's either a NOP or an ENDBR64, neither of which we care about
                        }
                        _ => panic!("Unrecognized instruction 0x0F{:02X}", instr2),
                    }
                }
                0x31 => {
                    // XOR r/m16/32/64, r16/32/64
                    self.modrm(
                        map,
                        if prefixes.contains(Prefixes::REX_W) {
                            OperandSize::R32
                        } else {
                            OperandSize::R64
                        },
                        |a, b, _| *a ^= b,
                    );
                }
                0x40 => prefixes |= Prefixes::REX,
                0x41 => prefixes |= Prefixes::REX | Prefixes::REX_B,
                0x42 => prefixes |= Prefixes::REX | Prefixes::REX_X,
                0x43 => prefixes |= Prefixes::REX | Prefixes::REX_X | Prefixes::REX_B,
                0x44 => prefixes |= Prefixes::REX | Prefixes::REX_R,
                0x45 => prefixes |= Prefixes::REX | Prefixes::REX_R | Prefixes::REX_B,
                0x46 => prefixes |= Prefixes::REX | Prefixes::REX_R | Prefixes::REX_X,
                0x47 => {
                    prefixes |= Prefixes::REX | Prefixes::REX_R | Prefixes::REX_X | Prefixes::REX_B
                }
                0x48 => prefixes |= Prefixes::REX | Prefixes::REX_W,
                0x49 => prefixes |= Prefixes::REX | Prefixes::REX_W | Prefixes::REX_B,
                0x4A => prefixes |= Prefixes::REX | Prefixes::REX_W | Prefixes::REX_X,
                0x4B => {
                    prefixes |= Prefixes::REX | Prefixes::REX_W | Prefixes::REX_X | Prefixes::REX_B
                }
                0x4C => prefixes |= Prefixes::REX | Prefixes::REX_W | Prefixes::REX_R,
                0x4D => {
                    prefixes |= Prefixes::REX | Prefixes::REX_W | Prefixes::REX_R | Prefixes::REX_B
                }
                0x4E => {
                    prefixes |= Prefixes::REX | Prefixes::REX_W | Prefixes::REX_R | Prefixes::REX_X
                }
                0x4F => {
                    prefixes |= Prefixes::REX
                        | Prefixes::REX_W
                        | Prefixes::REX_R
                        | Prefixes::REX_X
                        | Prefixes::REX_B
                }
                0x66 => prefixes |= Prefixes::OPSIZE,
                0x89 => {
                    // MOV r/m16/32/64, r16/32/64
                    self.modrm(
                        map,
                        if prefixes.contains(Prefixes::REX_W) {
                            OperandSize::R32
                        } else {
                            OperandSize::R64
                        },
                        |a, b, _| *a = b,
                    );
                }
                0xF3 => prefixes |= Prefixes::REP, // REP / REPZ
                _ => panic!("Unrecognized instruction {:#04X}", instr),
            }
        }
    }
}
