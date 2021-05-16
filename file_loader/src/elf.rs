use crate::{FileLoader, MemoryMap, ReadSeek};
use std::io::{Read, SeekFrom};

pub struct ElfFileLoader {}

pub struct ElfMemoryMap {}

impl FileLoader for ElfFileLoader {
    fn can_load(&self, file: &mut dyn ReadSeek) -> bool {
        let mut buf = vec![0u8; 6];
        if file.read_exact(&mut buf).is_err() {
            return false;
        }
        file.seek(SeekFrom::Start(0)).unwrap();
        matches!(&*buf, &[0x7F, b'E', b'L', b'F', _, 1])
    }

    fn load(&self, _file: &mut dyn Read) -> Box<dyn MemoryMap> {
        Box::new(ElfMemoryMap {})
    }
}

impl MemoryMap for ElfMemoryMap {}
