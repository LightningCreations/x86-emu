#![feature(iter_advance_by)]
use std::io::{Read, Seek};

pub mod elf;

use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub union XMMWord {
    single: [f32; 4],
    double: [f64; 2],
    i8: [u8; 16],
    i16: [u16; 8],
    i32: [u32; 4],
    i64: [u64; 2],
}

unsafe impl Zeroable for XMMWord {}
unsafe impl Pod for XMMWord {}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub union YMMWord {
    pub xmm: [XMMWord; 2],
    pub single: [f32; 8],
    pub double: [f64; 4],
    pub i8: [u8; 32],
    pub i16: [u16; 16],
    pub i32: [u32; 8],
    pub i64: [u64; 4],
}

unsafe impl Zeroable for YMMWord {}
unsafe impl Pod for YMMWord {}

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]
pub struct Registers {
    pub gprs: [u64; 16],
    pub cr: [u64; 16],
    pub sr: [u16; 8],
    pub rip: u64,
    pub rflags: u64,
    pub ymm: [YMMWord; 16],
}

pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek + ?Sized> ReadSeek for T {}

pub trait MemoryMap {
    fn bits(&self) -> u8;
    fn read_u8(&self, addr: u64) -> u8;
    fn read_u16(&self, addr: u64) -> u16 {
        (self.read_u8(addr) as u16) | (self.read_u8(addr + 1) as u16) << 8
    }
    fn read_u32(&self, addr: u64) -> u32 {
        (self.read_u16(addr) as u32) | (self.read_u16(addr + 2) as u32) << 16
    }
    fn read_u64(&self, addr: u64) -> u64 {
        (self.read_u32(addr) as u64) | (self.read_u32(addr + 1) as u64) << 16
    }
    fn read_xmmword(&self, addr: u64) -> XMMWord {
        let x = [self.read_u64(addr), self.read_u64(addr + 8)];
        bytemuck::cast(x)
    }
    fn read_ymmword(&self, addr: u64) -> YMMWord {
        let x = [self.read_xmmword(addr), self.read_xmmword(addr + 8)];
        bytemuck::cast(x)
    }
    fn registers(&self) -> &Registers;
    fn registers_mut(&mut self) -> &mut Registers;
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
