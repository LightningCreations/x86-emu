use crate::{FileLoader, MemoryMap, ReadSeek};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io::SeekFrom;

pub struct ElfFileLoader {}

pub struct ElfMemoryMap {}

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
        file.seek(SeekFrom::Current(4)).unwrap(); // Must be valid in order for us to be here
        let format = file.read_u8().unwrap(); // 1 = 32, 2 = 64
        file.seek(SeekFrom::Current(1)).unwrap(); // Again, must be valid in order for us to be here
        let little_endian = file.read_u8().unwrap() == 1;
        let _kernel = file.read_u8().unwrap();
        let _abiver = file.read_u8().unwrap();
        file.seek(SeekFrom::Current(7)).unwrap(); // 0-filled
        let _type = if little_endian {
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

        Box::new(ElfMemoryMap {})
    }
}

impl MemoryMap for ElfMemoryMap {}
