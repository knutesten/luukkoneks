use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::fmt;
use std::fmt::{Error, Formatter};

#[derive(Clone, PartialEq, Eq)]
pub struct Memory {
    cartridge_mem: Vec<u8>,
    working_mem: Vec<u8>,
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        unimplemented!()
    }
}

impl Memory {
    pub fn init_from_rom(rom_name: &str) -> Result<Memory, io::Error> {
        let mut file = File::open(rom_name)?;
        let mut mem = Memory {
            cartridge_mem: Vec::new(),
            working_mem: vec![0; 8 * 1024],
        };
        file.read_to_end(&mut mem.cartridge_mem)?;
        Ok(mem)
    }

    pub fn init_empty_with_instruction(offset: usize, prog: &[u8]) -> Memory {
        let mut cart_mem = vec![0; 32 * 1024];

        for x in 0..prog.len() {
            cart_mem[x + offset] = prog[x]
        }

        Memory {
            cartridge_mem: cart_mem,
            working_mem: vec![0; 8 * 1024],
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        return match addr {
            0..=0x7FFF => self.cartridge_mem[addr as usize],
            0xC000..=0xDFFF => self.working_mem[(addr - 0xC000) as usize],
            default => panic!("Tried to read invalid memory {:#x}", addr)
        };
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xC000..=0xDFFF => self.working_mem[(addr - 0xC000) as usize] = data,
            default => panic!("Invalid addr to write memory {:#x}", addr)
        }
    }
}