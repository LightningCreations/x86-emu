use crate::ProcessorImplementation;
use bitflags::bitflags;
use file_loader::MemoryMap;

use bytemuck::Zeroable;
use file_loader::Registers;

#[derive(PartialEq)]
pub enum OperandSize {
    R8,
    R16,
    R32,
    R64,
}

pub struct Amd64Interp {
    regs: Registers,
}

impl Amd64Interp {
    pub fn new() -> Amd64Interp {
        Amd64Interp {
            regs: Registers::zeroed(),
        }
    }

    pub fn modrm<T>(&mut self, map: &mut dyn MemoryMap, size: OperandSize, rex_r: bool, function: T)
    where
        T: Fn(u64, u64, OperandSize) -> u64,
    {
        let modrm_byte = map.read_u8(self.regs.rip);
        self.regs.rip += 1;
        let mut src =
            self.regs.gprs[(((modrm_byte >> 3) & 0x07) + if rex_r { 8 } else { 0 }) as usize];
        let saved;
        let dst = match modrm_byte & 0xC0 {
            0x00 => {
                saved = match modrm_byte & 7 {
                    4 => panic!("SIB not implemented!"),
                    5 => {
                        let offset = map.read_u32(self.regs.rip) as i64 as u64;
                        self.regs.rip += 4;
                        self.regs.rip + offset
                    }
                    _ => self.regs.gprs[(modrm_byte & 7) as usize],
                };
                map.read_u32(saved) as u64
            }
            0xC0 => {
                saved = (modrm_byte & 0x7) as u64;
                let result = &mut self.regs.gprs[saved as usize];
                if size == OperandSize::R32 {
                    *result &= 0x00000000FFFFFFFF; // Zero out higher half
                }
                *result
            }
            _ => panic!("Unrecognized ModR/M dst in {:#04X}", modrm_byte),
        };

        if size == OperandSize::R32 {
            src &= 0x00000000FFFFFFFF;
        }
        let result = function(dst, src, size);
        match modrm_byte & 0xC0 {
            0x00 => map.write_u32(saved, result as u32),
            0xC0 => self.regs.gprs[saved as usize] = result,
            _ => panic!("Unrecognized ModR/M dst in {:#04X}", modrm_byte),
        }
    }

    pub fn modrmdet<T>(&mut self, map: &mut dyn MemoryMap, size: OperandSize, mut function: T)
    where
        T: FnMut(u64, u8, OperandSize) -> u64,
    {
        let modrm_byte = map.read_u8(self.regs.rip);
        self.regs.rip += 1;
        let determ = ((modrm_byte >> 3) & 0x07) as u8;
        let saved;
        let dst = match modrm_byte & 0xC0 {
            0x00 => {
                saved = match modrm_byte & 7 {
                    4 => panic!("SIB not implemented!"),
                    5 => {
                        let offset = map.read_u32(self.regs.rip) as i64 as u64;
                        self.regs.rip += 4;
                        self.regs.rip + offset
                    }
                    _ => self.regs.gprs[(modrm_byte & 7) as usize],
                };
                map.read_u32(saved) as u64
            }
            0xC0 => {
                saved = (modrm_byte & 0x7) as u64;
                let result = &mut self.regs.gprs[saved as usize];
                if size == OperandSize::R32 {
                    *result &= 0x00000000FFFFFFFF; // Zero out higher half
                }
                *result
            }
            _ => panic!("Unrecognized ModR/M dst in {:#04X}", modrm_byte),
        };
        let result = function(dst, determ, size);
        match modrm_byte & 0xC0 {
            0x00 => map.write_u32(saved, result as u32),
            0xC0 => self.regs.gprs[saved as usize] = result,
            _ => panic!("Unrecognized ModR/M dst in {:#04X}", modrm_byte),
        }
    }

    pub fn modrmimm<T>(&mut self, map: &mut dyn MemoryMap, size: OperandSize, function: T)
    where
        T: Fn(u64, u8, i8, OperandSize) -> u64,
    {
        let modrm_byte = map.read_u8(self.regs.rip);
        self.regs.rip += 1;
        let determ = ((modrm_byte >> 3) & 0x07) as u8;
        let dst = match modrm_byte & 0xC0 {
            0xC0 => {
                let result = &mut self.regs.gprs[(modrm_byte & 0x7) as usize];
                if size == OperandSize::R32 {
                    *result &= 0x00000000FFFFFFFF; // Zero out higher half
                }
                *result
            }
            _ => panic!("Unrecognized ModR/M dst in {:#04X}", modrm_byte),
        };
        let imm = map.read_u8(self.regs.rip) as i8;
        self.regs.rip += 1;
        let result = function(dst, determ, imm, size);
        match modrm_byte & 0xC0 {
            0xC0 => self.regs.gprs[(modrm_byte & 0x7) as usize] = result,
            _ => panic!("Unrecognized ModR/M dst in {:#04X}", modrm_byte),
        }
    }

    pub fn modrmlea(&mut self, map: &mut dyn MemoryMap, size: OperandSize, rex_r: bool) {
        let modrm_byte = map.read_u8(self.regs.rip);
        self.regs.rip += 1;

        let result = match modrm_byte & 0xC0 {
            0x00 => match modrm_byte & 7 {
                4 => panic!("SIB not implemented!"),
                5 => {
                    let offset = map.read_u32(self.regs.rip) as i64 as u64;
                    self.regs.rip += 4;
                    self.regs.rip + offset
                }
                _ => self.regs.gprs[(modrm_byte & 7) as usize],
            },
            0xC0 => panic!("Can't LEA a register!"),
            _ => panic!("Unrecognized ModR/M dst in {:#04X}", modrm_byte),
        };

        let dst =
            &mut self.regs.gprs[(((modrm_byte >> 3) & 0x07) + if rex_r { 8 } else { 0 }) as usize];

        *dst = result;

        if size == OperandSize::R32 {
            *dst &= 0x00000000FFFFFFFF;
        }
    }
}

impl Default for Amd64Interp {
    fn default() -> Self {
        Self::new()
    }
}

bitflags! {
    struct Prefixes: u64 {
        const NONE = 0b00000000;
        const REP = 0b00000001;
        const OPSIZE = 0b00000010;
        const REX = 0b00000100;
        const REX_B = 0b00001000;
        const REX_X = 0b00010000;
        const REX_R = 0b00100000;
        const REX_W = 0b01000000;
    }
}

impl ProcessorImplementation for Amd64Interp {
    fn init(&mut self, map: &mut dyn MemoryMap) {
        self.regs.rip = map.entry_point();
        self.regs.gprs[4] = map.starting_stack();
    }
    fn running(&self) -> bool {
        true
    }
    fn tick(&mut self, map: &mut dyn MemoryMap) {
        let mut prefixes = Prefixes::NONE;
        let mut done = false;
        while !done {
            let instr = map.read_u8(self.regs.rip);
            self.regs.rip += 1;
            match instr {
                0x0F => {
                    let instr2 = map.read_u8(self.regs.rip);
                    self.regs.rip += 1;
                    match instr2 {
                        0x1E => {
                            let _ = map.read_u8(self.regs.rip); // ModR/M
                            self.regs.rip += 1;
                            done = true // It's either a NOP or an ENDBR64, neither of which we care about
                        }
                        _ => panic!("Unrecognized instruction 0x0F{:02X}", instr2),
                    }
                }
                0x31 => {
                    // XOR r/m16/32/64, r16/32/64
                    self.modrm(
                        map,
                        if prefixes.contains(Prefixes::REX_W) {
                            OperandSize::R32
                        } else {
                            OperandSize::R64
                        },
                        prefixes.contains(Prefixes::REX_R),
                        |a, b, _| a ^ b,
                    );
                }
                0x40 => prefixes |= Prefixes::REX,
                0x41 => prefixes |= Prefixes::REX | Prefixes::REX_B,
                0x42 => prefixes |= Prefixes::REX | Prefixes::REX_X,
                0x43 => prefixes |= Prefixes::REX | Prefixes::REX_X | Prefixes::REX_B,
                0x44 => prefixes |= Prefixes::REX | Prefixes::REX_R,
                0x45 => prefixes |= Prefixes::REX | Prefixes::REX_R | Prefixes::REX_B,
                0x46 => prefixes |= Prefixes::REX | Prefixes::REX_R | Prefixes::REX_X,
                0x47 => {
                    prefixes |= Prefixes::REX | Prefixes::REX_R | Prefixes::REX_X | Prefixes::REX_B
                }
                0x48 => prefixes |= Prefixes::REX | Prefixes::REX_W,
                0x49 => prefixes |= Prefixes::REX | Prefixes::REX_W | Prefixes::REX_B,
                0x4A => prefixes |= Prefixes::REX | Prefixes::REX_W | Prefixes::REX_X,
                0x4B => {
                    prefixes |= Prefixes::REX | Prefixes::REX_W | Prefixes::REX_X | Prefixes::REX_B
                }
                0x4C => prefixes |= Prefixes::REX | Prefixes::REX_W | Prefixes::REX_R,
                0x4D => {
                    prefixes |= Prefixes::REX | Prefixes::REX_W | Prefixes::REX_R | Prefixes::REX_B
                }
                0x4E => {
                    prefixes |= Prefixes::REX | Prefixes::REX_W | Prefixes::REX_R | Prefixes::REX_X
                }
                0x4F => {
                    prefixes |= Prefixes::REX
                        | Prefixes::REX_W
                        | Prefixes::REX_R
                        | Prefixes::REX_X
                        | Prefixes::REX_B
                }
                0x50 => {
                    self.regs.gprs[4] -= 8;
                    map.write_u64(
                        self.regs.gprs[4], /* rsp */
                        self.regs.gprs[0], /* rax */
                    );
                }
                0x54 => {
                    self.regs.gprs[4] -= 8;
                    map.write_u64(
                        self.regs.gprs[4], /* rsp */
                        self.regs.gprs[4], /* rsp */
                    );
                }
                0x5E => {
                    self.regs.gprs[6] /* rsi */ = map.read_u64(self.regs.gprs[4] /* rsp */);
                    self.regs.gprs[4] += 8;
                }
                0x66 => prefixes |= Prefixes::OPSIZE,
                0x83 => {
                    self.modrmimm(
                        map,
                        if prefixes.contains(Prefixes::REX_W) {
                            OperandSize::R32
                        } else {
                            OperandSize::R64
                        },
                        |a, determ, b, _| match determ {
                            0 => a + b as u64,
                            1 => a | b as u64,
                            2 => a + b as u64,
                            3 => a - b as u64,
                            4 => a & b as u64,
                            5 => a - b as u64,
                            6 => a ^ b as u64,
                            7 => a,
                            _ => unreachable!("determ should be between 0 and 7 inclusive"),
                        },
                    );
                }
                0x89 => {
                    // MOV r/m16/32/64, r16/32/64
                    self.modrm(
                        map,
                        if prefixes.contains(Prefixes::REX_W) {
                            OperandSize::R32
                        } else {
                            OperandSize::R64
                        },
                        prefixes.contains(Prefixes::REX_R),
                        |_, b, _| b,
                    );
                }
                0x8D => self.modrmlea(
                    map,
                    if prefixes.contains(Prefixes::REX_W) {
                        OperandSize::R32
                    } else {
                        OperandSize::R64
                    },
                    prefixes.contains(Prefixes::REX_R),
                ),
                0xF3 => prefixes |= Prefixes::REP, // REP / REPZ
                0xFF => {
                    let mut addr_real = 0;
                    let mut det_real = 0;
                    self.modrmdet(
                        map,
                        if prefixes.contains(Prefixes::REX_W) {
                            OperandSize::R32
                        } else {
                            OperandSize::R64
                        },
                        |a, determ, _| match determ {
                            0 => a + 1,
                            1 => a - 1,
                            x if x <= 7 => {
                                addr_real = a;
                                det_real = determ;
                                a
                            }
                            _ => unreachable!("determ should be between 0 and 7 inclusive"),
                        },
                    );
                    match det_real {
                        0 | 1 => {} // Already handled
                        2 => {
                            self.regs.gprs[4] -= 8;
                            map.write_u64(self.regs.gprs[4] /* rsp */, self.regs.rip);
                            self.regs.rip = addr_real;
                        }
                        x if x <= 7 => panic!("AAAA FF "),
                        _ => unreachable!("determ should be between 0 and 7 inclusive"),
                    }
                }
                _ => panic!("Unrecognized instruction {:#04X} at address {:#016X}", instr, self.regs.rip),
            }
        }
    }
}
