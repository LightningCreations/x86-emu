use crate::ProcessorImplementation;
use bitflags::bitflags;
use file_loader::MemoryMap;

use bytemuck::Zeroable;
use file_loader::Registers;

pub struct Amd64Interp {
    regs: Registers,
}

impl Amd64Interp {
    pub fn new() -> Amd64Interp {
        Amd64Interp {
            regs: Registers::zeroed(),
        }
    }
}

bitflags! {
    struct Prefixes: u64 {
        const NONE = 0b0000;
        const REP = 0b0001;
    }
}

impl ProcessorImplementation for Amd64Interp {
    fn init(&mut self, mm: &mut dyn MemoryMap) {
        self.regs.rip = mm.entry_point();
    }
    fn running(&self) -> bool {
        true
    }
    fn tick(&mut self, map: &mut dyn MemoryMap) {
        let mut prefixes = Prefixes::NONE;
        let mut done = false;
        while !done {
            let increment_ip = true;
            let instr = map.read_u8(self.regs.rip);
            match instr {
                0x0F => {
                    self.regs.rip += 1;
                    let instr2 = map.read_u8(self.regs.rip);
                    match instr2 {
                        0x1E => {
                            self.regs.rip += 1;
                            let _ = map.read_u8(self.regs.rip); // ModR/M
                            done = true // It's either a NOP or an ENDBR64, neither of which we care about
                        }
                        _ => panic!("Unrecognized instruction 0x0F{:02X}", instr2),
                    }
                }
                0xF3 => prefixes |= Prefixes::REP,
                _ => panic!("Unrecognized instruction {:#04X}", instr),
            }
            if increment_ip {
                self.regs.rip += 1;
            }
        }
    }
}
