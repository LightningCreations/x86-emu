#![feature(iter_advance_by)]
use std::io::{Read, Seek};

pub mod elf;

pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek + ?Sized> ReadSeek for T {}

pub trait MemoryMap {
    fn bits(&self) -> u8;
    fn entry_point(&self) -> u64;
    fn read_u8(&self, addr: u64) -> u8;
}

pub trait FileLoader {
    fn can_load(&self, file: &mut dyn ReadSeek) -> bool;
    fn load(&self, file: &mut dyn ReadSeek) -> Box<dyn MemoryMap>;
}

const LOADERS: [&dyn FileLoader; 1] = [&elf::ElfFileLoader {}];

pub trait LoadableFile: ReadSeek {
    fn loader(&mut self) -> Option<&'static dyn FileLoader>;
}

impl<F: ReadSeek> LoadableFile for F {
    fn loader(&mut self) -> Option<&'static dyn FileLoader> {
        for loader in LOADERS {
            if loader.can_load(self) {
                return Some(loader);
            }
        }
        None
    }
}
