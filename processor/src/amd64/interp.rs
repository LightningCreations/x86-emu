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

    pub fn modrm<T> (&mut self, map: &mut dyn MemoryMap, size: OperandSize, function: T) where T: Fn(&mut u64, u64, OperandSize) {
        let modrm_byte = map.read_u8(self.regs.rip);
        self.regs.rip += 1;
        let dst = match modrm_byte & 0xC0 {
            0xC0 => {
                let result = &mut self.regs.gprs[(modrm_byte & 0x7) as usize];
                if size == OperandSize::R32 {
                    *result = *result & 0x00000000FFFFFFFF; // Zero out higher half
                }
                result
            }
            _ => panic!("Unrecognized ModR/M dst in {:#04X}", modrm_byte)
        };
        let mut src = self.regs.gprs[((modrm_byte >> 3) & 0x07) as usize];
        if size == OperandSize::R32 {
            src = src & 0x00000000FFFFFFFF;
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
        const NONE = 0b0000;
        const REP = 0b0001;
        const OPSIZE = 0b0010;
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
                0x31 => { // XOR r/m16/32/64 r16/32/64
                    self.modrm(map, OperandSize::R32, |a, b, _| *a = *a ^ b);
                }
                0x66 => prefixes |= Prefixes::OPSIZE,
                0xF3 => prefixes |= Prefixes::REP, // REP / REPZ
                _ => panic!("Unrecognized instruction {:#04X}", instr),
            }
        }
    }
}
