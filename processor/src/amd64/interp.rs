use crate::ProcessorImplementation;
use file_loader::MemoryMap;

pub struct Amd64Interp {}

impl Amd64Interp {
    pub fn new() -> Amd64Interp {
        Amd64Interp {}
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
        match map.read_u8(regs.rip) {
            _ => panic!("Unrecognized instruction!"),
        }
    }
}
