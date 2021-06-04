use crate::{FileLoader, MemoryMap, ReadSeek, Registers, Regs64};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
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
    pub regs: Registers,
}

impl ElfFile {
    pub fn load(file: &mut dyn ReadSeek) -> ElfFile {
        file.seek(SeekFrom::Current(4)).unwrap(); // Must be valid in order for us to be here
        let format = file.read_u8().unwrap(); // 1 = 32, 2 = 64
        file.seek(SeekFrom::Current(1)).unwrap(); // Again, must be valid in order for us to be here
        let little_endian = file.read_u8().unwrap() == 1;
        let abi = file.read_u8().unwrap();
        let abiver = file.read_u8().unwrap();
        file.seek(SeekFrom::Current(7)).unwrap(); // 0-filled
        let bin_type = if little_endian {
            file.read_u16::<LittleEndian>().unwrap()
        } else {
            file.read_u16::<BigEndian>().unwrap()
        };
        let isa = if little_endian {
            file.read_u16::<LittleEndian>().unwrap()
        } else {
            file.read_u16::<BigEndian>().unwrap()
        };
        if (isa == 0x3 && format != 1) || (isa == 0x3E && format != 2) {
            panic!("ELF e_format and e_machine disagree");
        }
        if isa != 0x3 && isa != 0x3E {
            panic!("not an x86(_64) executable")
        }
        let version = if little_endian {
            file.read_u32::<LittleEndian>().unwrap()
        } else {
            file.read_u32::<BigEndian>().unwrap()
        };
        if version != 1 {
            panic!("incorrect ELF version");
        }
        let entry: u64 = if format == 1 {
            if little_endian {
                file.read_u32::<LittleEndian>().unwrap() as u64
            } else {
                file.read_u32::<BigEndian>().unwrap() as u64
            }
        } else if little_endian {
            file.read_u64::<LittleEndian>().unwrap()
        } else {
            file.read_u64::<BigEndian>().unwrap()
        };
        let phoff: u64 = if format == 1 {
            if little_endian {
                file.read_u32::<LittleEndian>().unwrap() as u64
            } else {
                file.read_u32::<BigEndian>().unwrap() as u64
            }
        } else if little_endian {
            file.read_u64::<LittleEndian>().unwrap()
        } else {
            file.read_u64::<BigEndian>().unwrap()
        };
        let _shoff: u64 = if format == 1 {
            if little_endian {
                file.read_u32::<LittleEndian>().unwrap() as u64
            } else {
                file.read_u32::<BigEndian>().unwrap() as u64
            }
        } else if little_endian {
            file.read_u64::<LittleEndian>().unwrap()
        } else {
            file.read_u64::<BigEndian>().unwrap()
        };
        let flags = if little_endian {
            file.read_u32::<LittleEndian>().unwrap()
        } else {
            file.read_u32::<BigEndian>().unwrap()
        };
        let _ehsize = if little_endian {
            file.read_u16::<LittleEndian>().unwrap()
        } else {
            file.read_u16::<BigEndian>().unwrap()
        };
        let _phentsize = if little_endian {
            file.read_u16::<LittleEndian>().unwrap()
        } else {
            file.read_u16::<BigEndian>().unwrap()
        };
        let phnum = if little_endian {
            file.read_u16::<LittleEndian>().unwrap()
        } else {
            file.read_u16::<BigEndian>().unwrap()
        };

        let mut next_off = phoff;
        let mut prghead = Vec::new();
        for _ in 0..phnum {
            file.seek(SeekFrom::Start(next_off)).unwrap();
            let ph_type = if little_endian {
                file.read_u32::<LittleEndian>().unwrap()
            } else {
                file.read_u32::<BigEndian>().unwrap()
            };
            let mut flags = if format == 1 {
                0
            } else if little_endian {
                file.read_u32::<LittleEndian>().unwrap()
            } else {
                file.read_u32::<BigEndian>().unwrap()
            };
            let offset = if format == 1 {
                if little_endian {
                    file.read_u32::<LittleEndian>().unwrap() as u64
                } else {
                    file.read_u32::<BigEndian>().unwrap() as u64
                }
            } else if little_endian {
                file.read_u64::<LittleEndian>().unwrap()
            } else {
                file.read_u64::<BigEndian>().unwrap()
            };
            let vaddr = if format == 1 {
                if little_endian {
                    file.read_u32::<LittleEndian>().unwrap() as u64
                } else {
                    file.read_u32::<BigEndian>().unwrap() as u64
                }
            } else if little_endian {
                file.read_u64::<LittleEndian>().unwrap()
            } else {
                file.read_u64::<BigEndian>().unwrap()
            };
            let paddr = if format == 1 {
                if little_endian {
                    file.read_u32::<LittleEndian>().unwrap() as u64
                } else {
                    file.read_u32::<BigEndian>().unwrap() as u64
                }
            } else if little_endian {
                file.read_u64::<LittleEndian>().unwrap()
            } else {
                file.read_u64::<BigEndian>().unwrap()
            };
            let filesz = if format == 1 {
                if little_endian {
                    file.read_u32::<LittleEndian>().unwrap() as u64
                } else {
                    file.read_u32::<BigEndian>().unwrap() as u64
                }
            } else if little_endian {
                file.read_u64::<LittleEndian>().unwrap()
            } else {
                file.read_u64::<BigEndian>().unwrap()
            };
            let memsz = if format == 1 {
                if little_endian {
                    file.read_u32::<LittleEndian>().unwrap() as u64
                } else {
                    file.read_u32::<BigEndian>().unwrap() as u64
                }
            } else if little_endian {
                file.read_u64::<LittleEndian>().unwrap()
            } else {
                file.read_u64::<BigEndian>().unwrap()
            };
            if format == 1 {
                flags = if little_endian {
                    file.read_u32::<LittleEndian>().unwrap()
                } else {
                    file.read_u32::<BigEndian>().unwrap()
                }
            }
            let align = if format == 1 {
                if little_endian {
                    file.read_u32::<LittleEndian>().unwrap() as u64
                } else {
                    file.read_u32::<BigEndian>().unwrap() as u64
                }
            } else if little_endian {
                file.read_u64::<LittleEndian>().unwrap()
            } else {
                file.read_u64::<BigEndian>().unwrap()
            };

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

        let regs64 = Regs64 {
            r: [0; 16],
            rip: entry,
        };

        let regs = Registers {
            regs64: Some(regs64),
        };

        ElfFile {
            bits: if format == 1 { 32 } else { 64 },
            abi,
            abiver,
            bin_type,
            isa,
            flags,
            prghead,
            secthead: Vec::new(),
            regs,
        }
    }
}

pub struct ElfMemoryMap {
    prghead: Vec<ElfPrgHead>,
    bits: u8,
    regs: Registers,
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

    fn registers(&self) -> &Registers {
        &self.regs
    }

    fn registers_mut(&mut self) -> &mut Registers {
        &mut self.regs
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
            regs: elffile.regs,
        })
    }
}
