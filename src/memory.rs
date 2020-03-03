use std::fs::File;
use std::io::prelude::*;

pub struct Memory {
    cartridge_mem: Vec<u8>,
    working_mem: Vec<u8>
}

impl Memory {
    pub fn init_from_rom(rom_name: string) -> Memory {
        let mut file = File::open("Tetris.gb")?;
        let mem = Memory {
            cartridge_mem: Vec::new(),
            working_mem: Vec::with_capacity(8*1024)
        };
        file.read_to_end(&mut mem.cartridge_mem)?;
        mem
    }

    fn read_memory(self, addr: u16) -> u8 {
        return match addr {
            0..0x7FFF => self.cartridge_mem[addr],
            0xC000..0xDFFF => self.working_mem[addr-0xC000],
            default => panic!("Tried to read invalid memory {:#x}", addr)
        }
    }

    fn write_memory(self, addr: u16, data: u8) {
        match(addr) {
            0xC000..0xDFFF => self.working_mem[addr-0xC000] = data,
            default=> panic!("Invalid addr to write memory {:#x}", addr)
        }
    }
}