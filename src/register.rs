use std::fmt;
use std::fmt::{Error, Formatter};

pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
}

impl fmt::Debug for Registers {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        writeln!(f,
                 "AF={:#06x}\nBC={:#06x}\nDE={:#06x}\nHL={:#06x}\nSP={:#06x}\nPC={:#06x}",
                 self.get_af(),
                 self.get_bc(),
                 self.get_de(),
                 self.get_hl(),
                 self.sp,
                 self.pc)
    }
}

const Z_FLAG: u8 = 0b10000000;
const N_FLAG: u8 = 0b01000000;
const H_FLAG: u8 = 0b00100000;
const C_FLAG: u8 = 0b00010000;

macro_rules! getter {
    ($name:ident, $reg1:ident, $reg2:ident) => {
        pub fn $name(&self) -> u16 {
            return ((self.$reg1 as u16) << 8) | self.$reg2 as u16;
        }
    }
}

macro_rules! setter {
    ($name:ident, $reg1:ident, $reg2:ident) => {
        pub fn $name(&mut self, value: u16) {
            self.$reg1 = (value >> 8) as u8;
            self.$reg2 = value as u8;
        }
    }
}

macro_rules! combo_reg {
    ($getName:ident, $setName:ident, $reg1:ident, $reg2:ident) => {
        getter!($getName, $reg1, $reg2);
        setter!($setName, $reg1, $reg2);
    }
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0x01,
            f: 0xB0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            pc: 0x0100,
            sp: 0xFFFE,
        }
    }

    combo_reg!(get_af, set_af, a, f);
    combo_reg!(get_bc, set_bc, b, c);
    combo_reg!(get_de, set_de, d, e);
    combo_reg!(get_hl, set_hl, h, l);
}
