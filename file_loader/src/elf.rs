use crate::{FileLoader, MemoryMap, ReadSeek};
use crate::{XMMWord, YMMWord};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::SeekFrom;
use std::vec::Vec;

struct ElfPrgHead {
    pub ph_type: u32,
    pub flags: u32,
    pub vaddr: u64,
    pub paddr: u64,
    pub memsz: u64,
    pub align: u64,
    pub data: Vec<u8>,
}

struct ElfSectHead {}

struct ElfFile {
    pub bits: u8,
    pub abi: u8,
    pub abiver: u8,
    pub bin_type: u16,
    pub isa: u16,
    pub flags: u32,
    pub prghead: Vec<ElfPrgHead>,
    pub secthead: Vec<ElfSectHead>,
    pub e_entry: u64,
}

impl ElfFile {
    pub fn load(file: &mut dyn ReadSeek) -> ElfFile {
        file.seek(SeekFrom::Current(4)).unwrap(); // Must be valid in order for us to be here
        let format = file.read_u8().unwrap(); // 1 = 32, 2 = 64
        file.seek(SeekFrom::Current(1)).unwrap(); // Again, must be valid in order for us to be here
        let little_endian = file.read_u8().unwrap() == 1;
        if !little_endian {
            panic!("x86 is little-endian, this ain't");
        }
        let abi = file.read_u8().unwrap();
        let abiver = file.read_u8().unwrap();
        file.seek(SeekFrom::Current(7)).unwrap(); // 0-filled
        let read_u32_u64 = |file: &mut dyn ReadSeek| {
            if format == 1 {
                file.read_u32::<LittleEndian>().unwrap() as u64
            } else {
                file.read_u64::<LittleEndian>().unwrap()
            }
        };
        let bin_type = file.read_u16::<LittleEndian>().unwrap();
        let isa = file.read_u16::<LittleEndian>().unwrap();
        if (isa == 0x3 && format != 1) || (isa == 0x3E && format != 2) {
            panic!("ELF e_format and e_machine disagree");
        }
        if isa != 0x3 && isa != 0x3E {
            panic!("not an x86(_64) executable")
        }
        let version = file.read_u32::<LittleEndian>().unwrap();
        if version != 1 {
            panic!("incorrect ELF version");
        }
        let entry = read_u32_u64(file);
        let phoff = read_u32_u64(file);
        let _shoff = read_u32_u64(file);
        let flags = file.read_u32::<LittleEndian>().unwrap();
        let _ehsize = file.read_u16::<LittleEndian>().unwrap();
        let _phentsize = file.read_u16::<LittleEndian>().unwrap();
        let phnum = file.read_u16::<LittleEndian>().unwrap();
        let mut next_off = phoff;
        let mut prghead = Vec::new();
        for _ in 0..phnum {
            file.seek(SeekFrom::Start(next_off)).unwrap();
            let ph_type = file.read_u32::<LittleEndian>().unwrap();
            let mut flags = 0; // Because... AAAA
            if format != 1 {
                flags = file.read_u32::<LittleEndian>().unwrap();
            }
            let offset = read_u32_u64(file);
            let vaddr = read_u32_u64(file);
            let paddr = read_u32_u64(file);
            let filesz = read_u32_u64(file);
            let memsz = read_u32_u64(file);
            if format == 1 {
                flags = file.read_u32::<LittleEndian>().unwrap();
            }
            let align = read_u32_u64(file);

            next_off = file.seek(SeekFrom::Current(0)).unwrap();
            file.seek(SeekFrom::Start(offset)).unwrap();
            let mut data = vec![0; filesz as usize];
            file.read_exact(&mut data).unwrap();

            prghead.push(ElfPrgHead {
                ph_type,
                flags,
                vaddr,
                paddr,
                memsz,
                align,
                data,
            });
        }
        let ram = vec![0; 0x10000]; // 64K of RAM. All ya need, right?
        prghead.push(ElfPrgHead {
            ph_type: 0xFFFFFFFF, // psuedo-type
            flags: 3,            // RW
            vaddr: 0x7FFF0000,
            paddr: 0, // paddr is unused on x86
            memsz: 0x10000,
            align: 1, // Don't care
            data: ram,
        });
        ElfFile {
            bits: if format == 1 { 32 } else { 64 },
            abi,
            abiver,
            bin_type,
            isa,
            flags,
            prghead,
            secthead: Vec::new(),
            e_entry: entry,
        }
    }
}

use std::borrow::Cow;

pub fn prefix_zeroed<const N: usize>(slice: &[u8]) -> Cow<[u8; N]> {
    slice
        .get(..N)
        .map(|s| &bytemuck::cast_slice(s)[0])
        .map(Cow::Borrowed)
        .unwrap_or_else(|| {
            let mut x = [0u8; { N }];
            x[..slice.len()].copy_from_slice(slice);
            Cow::Owned(x)
        })
}

pub struct ElfMemoryMap {
    prghead: Vec<ElfPrgHead>,
    bits: u8,
    e_entry: u64,
}

impl MemoryMap for ElfMemoryMap {
    fn bits(&self) -> u8 {
        self.bits
    }

    fn read_u8(&self, addr: u64) -> u8 {
        for ph in &self.prghead {
            if ph.vaddr <= addr && (ph.vaddr + ph.memsz) > addr {
                return *ph.data.get((addr - ph.vaddr) as usize).unwrap_or(&0);
            }
        }
        panic!("Segmentation fault")
    }

    fn read_u16(&self, addr: u64) -> u16 {
        for ph in &self.prghead {
            if ph.vaddr <= addr && (ph.vaddr + ph.memsz) > (addr + 1) {
                return bytemuck::cast(*prefix_zeroed::<2>(
                    ph.data.get(((addr - ph.vaddr) as usize)..).unwrap_or(&[]),
                ));
            }
        }
        panic!("Segmentation fault")
    }

    fn read_u32(&self, addr: u64) -> u32 {
        for ph in &self.prghead {
            if ph.vaddr <= addr && (ph.vaddr + ph.memsz) > (addr + 1) {
                return bytemuck::cast(*prefix_zeroed::<4>(
                    ph.data.get(((addr - ph.vaddr) as usize)..).unwrap_or(&[]),
                ));
            }
        }
        panic!("Segmentation fault")
    }

    fn read_u64(&self, addr: u64) -> u64 {
        for ph in &self.prghead {
            if ph.vaddr <= addr && (ph.vaddr + ph.memsz) > (addr + 1) {
                return bytemuck::cast(*prefix_zeroed::<8>(
                    ph.data.get(((addr - ph.vaddr) as usize)..).unwrap_or(&[]),
                ));
            }
        }
        panic!("Segmentation fault")
    }

    fn read_xmmword(&self, addr: u64) -> XMMWord {
        for ph in &self.prghead {
            if ph.vaddr <= addr && (ph.vaddr + ph.memsz) > (addr + 1) {
                return bytemuck::cast(*prefix_zeroed::<16>(
                    ph.data.get(((addr - ph.vaddr) as usize)..).unwrap_or(&[]),
                ));
            }
        }
        panic!("Segmentation fault")
    }

    fn read_ymmword(&self, addr: u64) -> YMMWord {
        for ph in &self.prghead {
            if ph.vaddr <= addr && (ph.vaddr + ph.memsz) > (addr + 1) {
                return bytemuck::cast(*prefix_zeroed::<32>(
                    ph.data.get(((addr - ph.vaddr) as usize)..).unwrap_or(&[]),
                ));
            }
        }
        panic!("Segmentation fault")
    }

    fn write_u8(&mut self, addr: u64, data: u8) {
        for ph in &mut self.prghead {
            if ph.vaddr <= addr && (ph.vaddr + ph.memsz) > addr {
                ph.data[(addr - ph.vaddr) as usize] = data;
                return;
            }
        }
        panic!("Segmentation fault writing to address {:#018X}", addr)
    }

    fn entry_point(&self) -> u64 {
        self.e_entry
    }

    fn starting_stack(&self) -> u64 {
        0x7FFFFFF8 // Works for both 32-bit and 64-bit
    }
}

pub struct ElfFileLoader {}

impl FileLoader for ElfFileLoader {
    fn can_load(&self, file: &mut dyn ReadSeek) -> bool {
        let mut buf = vec![0u8; 7];
        if file.read_exact(&mut buf).is_err() {
            return false;
        }
        file.seek(SeekFrom::Start(0)).unwrap();
        matches!(&*buf, &[0x7F, b'E', b'L', b'F', _, _, 1])
    }

    fn load(&self, file: &mut dyn ReadSeek) -> Box<dyn MemoryMap> {
        let elffile = ElfFile::load(file);
        Box::new(ElfMemoryMap {
            prghead: elffile.prghead,
            bits: elffile.bits,
            e_entry: elffile.e_entry,
        })
    }
}
