use crate::{FileLoader, MemoryMap, ReadSeek};
use std::io::{Read, SeekFrom};

pub struct ElfFileLoader {}

pub struct ElfMemoryMap {}

impl FileLoader for ElfFileLoader {
    fn can_load(&self, file: &mut dyn ReadSeek) -> bool {
        let mut buf = vec![0u8; 6];
        match file.read_exact(&mut buf) {
            Err(_) => return false,
            _ => {}
        };
        file.seek(SeekFrom::Start(0)).unwrap();
        if let &[0x7F, b'E', b'L', b'F', _, 1] = &*buf {
            true
        } else {
            false
        }
    }

    fn load(&self, _file: &mut dyn Read) -> Box<dyn MemoryMap> {
        Box::new(ElfMemoryMap {})
    }
}

impl MemoryMap for ElfMemoryMap {}
