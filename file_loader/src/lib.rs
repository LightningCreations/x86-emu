#![feature(iter_advance_by)]
use std::io::{Read, Seek};

pub mod elf;

pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek + ?Sized> ReadSeek for T {}

pub trait MemoryMap {}

pub trait FileLoader {
    fn can_load(&self, file: &mut dyn ReadSeek) -> bool;
    fn load(&self, file: &mut dyn ReadSeek) -> Box<dyn MemoryMap>;
}

const LOADERS: [&dyn FileLoader; 1] = [&elf::ElfFileLoader {}];

pub trait LoadableFile: ReadSeek {
    fn get_loader(&mut self) -> Option<&'static dyn FileLoader>;
}

impl<F: ReadSeek> LoadableFile for F {
    fn get_loader(&mut self) -> Option<&'static dyn FileLoader> {
        for loader in LOADERS {
            if loader.can_load(self) {
                return Some(loader);
            }
        }
        None
    }
}
