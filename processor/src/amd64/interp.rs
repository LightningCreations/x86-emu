use bitflags::bitflags;
use crate::ProcessorImplementation;
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
        let regs = map.registers().regs64.as_ref().unwrap();
        let mut prefixes = Prefixes::NONE;
        let done = false;
        while !done {
            let increment_ip = true;
            let instr = map.read_u8(regs.rip);
            match instr {
                0xF3 => prefixes |= Prefixes::REP,
                _ => panic!("Unrecognized instruction {:#04X}", instr),
            }
            if increment_ip {
                map.registers_mut().regs64.unwrap().rip += 1;
            }
        }
    }
}
