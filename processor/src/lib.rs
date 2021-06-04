use file_loader::MemoryMap;

pub mod amd64;

pub trait ProcessorImplementation {
    fn init(&mut self, map: &mut dyn MemoryMap);
    fn running(&self) -> bool;
    fn tick(&mut self, map: &mut dyn MemoryMap);
}

pub trait ProcessableMemoryMap {
    fn processor_impl(&self) -> Option<Box<dyn ProcessorImplementation>>;
}

impl ProcessableMemoryMap for dyn MemoryMap {
    fn processor_impl(&self) -> Option<Box<dyn ProcessorImplementation>> {
        if self.bits() == 64 {
            Some(Box::new(amd64::interp::Amd64Interp::new()))
        } else {
            panic!("Non-64-bit architectures are not currently supported!")
        }
    }
}
