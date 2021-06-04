use crate::ProcessorImplementation;
use bitflags::bitflags;
use file_loader::MemoryMap;

pub struct Amd64Interp {}

impl Amd64Interp {
    pub fn new() -> Amd64Interp {
        Amd64Interp {}
    }
}

bitflags! {
    struct Prefixes: u64 {
        const NONE = 0b0000;
        const REP = 0b0001;
    }
}

impl ProcessorImplementation for Amd64Interp {
    fn init(&mut self, _: &mut dyn MemoryMap) {
        // No initialization needed for the interpreter
    }
    fn running(&self) -> bool {
        true
    }
    fn tick(&mut self, map: &mut dyn MemoryMap) {
        let mut prefixes = Prefixes::NONE;
        let mut done = false;
        while !done {
            let regs = map.registers();
            let increment_ip = true;
            let instr = map.read_u8(regs.rip);
            match instr {
                0x0F => {
                    map.registers_mut().rip += 1;
                    let regs = map.registers();
                    let instr2 = map.read_u8(regs.rip);
                    match instr2 {
                        0x1E => {
                            let _ = map.read_u8(regs.rip+1); // ModR/M
                            map.registers_mut().rip += 1;
                            done = true // It's either a NOP or an ENDBR64, neither of which we care about
                        }
                        _ => panic!("Unrecognized instruction 0x0F{:02X}", instr2),
                    }
                }
                0xF3 => prefixes |= Prefixes::REP,
                _ => panic!("Unrecognized instruction {:#04X}", instr),
            }
            if increment_ip {
                map.registers_mut().rip += 1;
            }
        }
    }
}
