#![feature(iter_advance_by)]
use std::io::{Read, Seek};

pub mod elf;

use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub union XMMReg {
    single: [f32; 4],
    double: [f64; 2],
    i8: [u8; 16],
    i16: [u16; 8],
    i32: [u32; 4],
    i64: [u64; 2],
}

unsafe impl Zeroable for XMMReg {}
unsafe impl Pod for XMMReg {}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub union YMMReg {
    pub xmm: [XMMReg; 2],
    pub single: [f32; 8],
    pub double: [f64; 4],
    pub i8: [u8; 32],
    pub i16: [u16; 16],
    pub i32: [u32; 8],
    pub i64: [u64; 4],
}

unsafe impl Zeroable for YMMReg {}
unsafe impl Pod for YMMReg {}

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]
pub struct Registers {
    pub gprs: [u64; 16],
    pub cr: [u64; 16],
    pub sr: [u16; 8],
    pub rip: u64,
    pub rflags: u64,
    pub ymm: [YMMReg; 16],
}

pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek + ?Sized> ReadSeek for T {}

pub trait MemoryMap {
    fn bits(&self) -> u8;
    fn read_u8(&self, addr: u64) -> u8;
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
