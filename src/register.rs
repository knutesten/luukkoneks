use std::fmt;
use std::fmt::{Error, Formatter};
use bitfield::bitfield;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum RegisterType {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
    PC,
    SP,
    AF,
    BC,
    DE,
    HL,
}

bitfield!{
    #[derive(PartialEq, Clone, Eq, Copy)]
    pub struct FLAGS(u8);
    impl Debug;
    pub get_z, set_z: 7;
    pub get_n, set_n: 6;
    pub get_h, set_h: 5;
    pub get_c, set_c: 4;
}

#[derive(PartialEq, Clone, Eq)]
pub struct Registers {
    pub a: u8,
    pub f: FLAGS,
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
                 "Flags={:#010b}\nAF={:#06x}\nBC={:#06x}\nDE={:#06x}\nHL={:#06x}\nSP={:#06x}\nPC={:#06x}",
                 self.f.0,
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
            f: FLAGS(0xB0),
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

    pub fn get_af(&self) -> u16 {
        return (((self.a as u16) << 8) | self.f.0 as u16) as u16;
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = FLAGS((value & 0xF0) as u8)
    }

    pub fn set_flags(&mut self, flags: FLAGS) {
        self.f = flags
    }

    pub fn get_flags(&self) -> FLAGS {
        return self.f
    }

    combo_reg!(get_bc, set_bc, b, c);
    combo_reg!(get_de, set_de, d, e);
    combo_reg!(get_hl, set_hl, h, l);

    pub fn set(&mut self, register_type: RegisterType, value: u16) {
        match register_type {
            RegisterType::A => { self.a = value as u8 }
            RegisterType::B => { self.b = value as u8 }
            RegisterType::C => { self.c = value as u8 }
            RegisterType::D => { self.d = value as u8 }
            RegisterType::E => { self.e = value as u8 }
            RegisterType::F => { self.f = FLAGS((value & 0xF0) as u8) }
            RegisterType::H => { self.h = value as u8 }
            RegisterType::L => { self.l = value as u8 }
            RegisterType::PC => { self.pc = value }
            RegisterType::SP => { self.sp = value }
            RegisterType::AF => { self.set_af(value) }
            RegisterType::BC => { self.set_bc(value) }
            RegisterType::DE => { self.set_de(value) }
            RegisterType::HL => { self.set_hl(value) }
        }
    }

    pub fn get(&self, register_type: RegisterType) -> u16 {
        return match register_type {
            RegisterType::A => { self.a as u16 }
            RegisterType::B => { self.b as u16 }
            RegisterType::C => { self.c as u16 }
            RegisterType::D => { self.d as u16 }
            RegisterType::E => { self.e as u16 }
            RegisterType::F => { self.f.0 as u16 }
            RegisterType::H => { self.h as u16 }
            RegisterType::L => { self.l as u16 }
            RegisterType::PC => { self.pc }
            RegisterType::SP => { self.sp }
            RegisterType::AF => { self.get_af() }
            RegisterType::BC => { self.get_bc() }
            RegisterType::DE => { self.get_de() }
            RegisterType::HL => { self.get_hl() }
        };
    }
}
