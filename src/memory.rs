use std::fs::File;
use std::io::prelude::*;
use std::io;

#[derive(Clone,Eq)]
pub struct Memory {
    cartridge_mem: Vec<u8>,
    working_mem: Vec<u8>
}

impl PartialEq for Memory {
    fn eq(&self, other: &Self) -> bool {
        self.cartridge_mem.eq(&other.cartridge_mem) &&
        self.working_mem.eq(&other.working_mem)
    }
}
impl Memory {
    pub fn init_from_rom(rom_name: &str) -> Result<Memory, io::Error> {
        let mut file = File::open(rom_name)?;
        let mut mem = Memory {
            cartridge_mem: Vec::new(),
            working_mem: Vec::with_capacity(8*1024)
        };
        file.read_to_end(&mut mem.cartridge_mem)?;
        Ok(mem)
    }

    fn read(self, addr: u16) -> u8 {
        return match addr {
            0..=0x7FFF => self.cartridge_mem[addr as usize],
            0xC000..=0xDFFF => self.working_mem[(addr-0xC000) as usize],
            default => panic!("Tried to read invalid memory {:#x}", addr)
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xC000..=0xDFFF => self.working_mem[(addr-0xC000) as usize] = data,
            default=> panic!("Invalid addr to write memory {:#x}", addr)
        }
    }
}