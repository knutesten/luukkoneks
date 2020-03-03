use std::thread::sleep;
use std::time::Duration;

use crate::memory::Memory;
use crate::register::{Registers, RegisterType};
use crate::register::RegisterType::*;

const CYCLE_DURATION: Duration = Duration::from_millis(10);

pub fn process_instruction(registers: &mut Registers, memory: &mut Memory) -> usize {
    let program_counter = registers.pc;
    let instruction = memory.read(program_counter);
    let cycles = match instruction {
        0x40..=0x7F => handle_load_instruction(instruction, registers, memory),
        _ => panic!("Unsupported instruction {}", instruction)
    };
    sleep(CYCLE_DURATION);
    cycles
}

fn handle_load_instruction(instruction: u8,
                           registers: &mut Registers,
                           memory: &mut Memory) -> usize {
    let to = match instruction {
        0x40..=0x47 => B,
        0x48..=0x4F => C,
        0x50..=0x57 => D,
        0x58..=0x5F => E,
        0x60..=0x67 => H,
        0x68..=0x6F => L,
        0x70..=0x77 => HL,
        0x78..=0x7F => A,
        _ => panic!("Unsupported load instruction {}", instruction)
    };

    let from = match instruction & 0xF {
        0x00 | 0x08 => B,
        0x01 | 0x09 => C,
        _ => panic!("Unsupported load instruction {}", instruction)
    };

    let value = read_value(from, { from == HL }, registers, memory);
    write_value(to, value, { to == HL }, registers, memory);

    if to == HL || from == HL { 8 } else { 4 }
}

fn write_value(to: RegisterType, value: u16, to_memory: bool,
               registers: &mut Registers, memory: &mut Memory) {
    if to_memory {
        memory.write(registers.get(to), value as u8);
    } else {
        registers.set(to, value);
    };
}

fn read_value(from: RegisterType, from_memory: bool,
              registers: &Registers, memory: &Memory) -> u16 {
    return if from_memory {
        memory.read(registers.get(from) as u16) as u16
    } else {
        registers.get(from)
    };
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! load_test_only_reg {
        ($instruction: tt, $to: tt, $from: tt, $cycles: tt) => {
            {
                let mut registers = Registers::new();
                registers.set($from, 0xff);
                let mut memory = Memory::init_empty_with_instruction(0x0100, &[$instruction]);

                let mut expected_registers = registers.clone();
                expected_registers.set($to, 0xff);
                let expected_memory = memory.clone();

                let cycles = process_instruction(&mut registers, &mut memory);
                assert_eq!($cycles, cycles);
                assert_eq!(expected_registers, registers);
                assert_eq!(expected_memory, memory);
            }
        }
    }

    #[test]
    fn test_x40() {
        load_test_only_reg!(0x40, B, B, 4)
    }

    #[test]
    fn test_x50() {
        load_test_only_reg!(0x50, D, B, 4)
    }

    #[test]
    fn test_x60() {
        load_test_only_reg!(0x60, H, B, 4)
    }

    #[test]
    fn test_x70() {
        let mut registers = Registers::new();
        registers.set(B, 0xff);
        registers.set(HL, 0xD000);
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0x70]);

        let mut expected_registers = registers.clone();
        let mut expected_memory = memory.clone();
        expected_memory.write(0xD000, 0xFF);

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(8, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }

    #[test]
    fn test_x48() {
        let mut registers = Registers::new();
        registers.set(B, 0xff);
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0x48]);

        let mut expected_registers = registers.clone();
        expected_registers.set(C, 0xff);
        let expected_memory = memory.clone();

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(4, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }

    #[test]
    fn test_x58() {
        let mut registers = Registers::new();
        registers.set(B, 0xff);
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0x58]);

        let mut expected_registers = registers.clone();
        expected_registers.set(E, 0xff);
        let expected_memory = memory.clone();

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(4, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }

    #[test]
    fn test_x68() {
        let mut registers = Registers::new();
        registers.set(B, 0xff);
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0x68]);

        let mut expected_registers = registers.clone();
        expected_registers.set(L, 0xff);
        let expected_memory = memory.clone();

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(4, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }

    #[test]
    fn test_x78() {
        let mut registers = Registers::new();
        registers.set(B, 0xff);
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0x78]);

        let mut expected_registers = registers.clone();
        expected_registers.set(A, 0xff);
        let expected_memory = memory.clone();

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(4, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }

    #[test]
    fn test_x41() {
        let mut registers = Registers::new();
        registers.set(C, 0xff);
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0x41]);

        let mut expected_registers = registers.clone();
        expected_registers.set(B, 0xff);
        let expected_memory = memory.clone();

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(4, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }

    #[test]
    fn test_x51() {
        let mut registers = Registers::new();
        registers.set(C, 0xff);
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0x51]);

        let mut expected_registers = registers.clone();
        expected_registers.set(D, 0xff);
        let expected_memory = memory.clone();

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(4, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }

    #[test]
    fn test_x61() {
        let mut registers = Registers::new();
        registers.set(C, 0xff);
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0x61]);

        let mut expected_registers = registers.clone();
        expected_registers.set(H, 0xff);
        let expected_memory = memory.clone();

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(4, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }

    #[test]
    fn test_x71() {
        let mut registers = Registers::new();
        registers.set(C, 0xff);
        registers.set(HL, 0xD000);
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0x71]);

        let mut expected_registers = registers.clone();
        let mut expected_memory = memory.clone();
        expected_memory.write(0xD000, 0xFF);

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(8, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }

    #[test]
    fn test_x49() {
        let mut registers = Registers::new();
        registers.set(C, 0xff);
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0x49]);

        let mut expected_registers = registers.clone();
        expected_registers.set(C, 0xff);
        let expected_memory = memory.clone();

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(4, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }

    #[test]
    fn test_x59() {
        let mut registers = Registers::new();
        registers.set(C, 0xff);
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0x59]);

        let mut expected_registers = registers.clone();
        expected_registers.set(E, 0xff);
        let expected_memory = memory.clone();

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(4, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }

    #[test]
    fn test_x69() {
        let mut registers = Registers::new();
        registers.set(C, 0xff);
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0x69]);

        let mut expected_registers = registers.clone();
        expected_registers.set(L, 0xff);
        let expected_memory = memory.clone();

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(4, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }

    #[test]
    fn test_x79() {
        let mut registers = Registers::new();
        registers.set(C, 0xff);
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0x79]);

        let mut expected_registers = registers.clone();
        expected_registers.set(A, 0xff);
        let expected_memory = memory.clone();

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(4, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }
}