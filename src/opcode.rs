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
        0x00 => handle_nop(registers),
        0x2F => handle_cpl(registers),
        0x05 | 0x15 | 0x25 | 0x35 => handle_dec_n(instruction, registers, memory),
        0x0B | 0x1B | 0x2B | 0x3B => handle_dec_nn(instruction, registers),
        0xC3 => handle_jump(&program_counter, registers, memory),
        0x40..=0x7F => handle_load(instruction, registers, memory),
        _ => panic!("Unsupported instruction {:#04x}", instruction)
    };
    sleep(CYCLE_DURATION);
    cycles
}

fn handle_dec_n(instruction: u8, registers: &mut Registers, memory: &mut Memory) -> usize {
    let target = match instruction {
        0x05 => B,
        0x15 => D,
        0x25 => H,
        0x35 => HL,
        _ => panic!("Unsupported dec instruction {:#04x}", instruction),
    };

    let current_value = read_value(target, target == HL, registers, memory);
    let new_value = current_value.wrapping_sub(1);
    write_value(target, new_value, HL == target, registers, memory);

    let mut flags = registers.get_flags();
    flags.set_z(0x00 == new_value);
    flags.set_n(true);
    flags.set_h(0x00 != (0xF & current_value));
    registers.set_flags(flags);

    increment_pc(registers);

    if HL == target { 12 } else { 4 }
}

fn handle_dec_nn(instruction: u8, registers: &mut Registers) -> usize {
    let target = match instruction {
        0x0B => BC,
        0x1B => DE,
        0x2B => HL,
        0x3B => SP,
        _ => panic!("Unsupported dec instruction {:#04x}", instruction),
    };

    registers.set(target, registers.get(target).wrapping_sub(1));
    increment_pc(registers);
    8
}

fn handle_cpl(registers: &mut Registers) -> usize {
    registers.set(A, !registers.get(A));
    let mut flags = registers.get_flags();
    flags.set_n(true);
    flags.set_h(true);
    registers.set_flags(flags);
    increment_pc(registers);
    4
}

fn handle_jump(program_counter: &u16, registers: &mut Registers, memory: &Memory) -> usize {
    let pc = ((memory.read(program_counter + 1) as u16) << 8) +
        memory.read(program_counter + 2) as u16;
    registers.set(PC, pc);
    16
}

fn handle_nop(registers: &mut Registers) -> usize {
    increment_pc(registers);
    4
}

fn handle_load(instruction: u8,
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
        _ => panic!("Unsupported load instruction {:#04x}", instruction)
    };

    let from = match instruction & 0xF {
        0x00 | 0x08 => B,
        0x01 | 0x09 => C,
        0x02 | 0x0A => D,
        _ => panic!("Unsupported load instruction {:#04x}", instruction)
    };

    let value = read_value(from, { from == HL }, registers, memory);
    write_value(to, value, { to == HL }, registers, memory);
    increment_pc(registers);

    if to == HL || from == HL { 8 } else { 4 }
}

fn increment_pc(registers: &mut Registers) {
    registers.set(PC, 1 + registers.get(PC));
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
        ($instruction: tt, $to: tt, $from: tt) => {
            {
                let mut registers = Registers::new();
                registers.set($from, 0xff);
                let mut memory = Memory::init_empty_with_instruction(0x0100, &[$instruction]);

                let mut expected_registers = registers.clone();
                expected_registers.set($to, 0xff);
                expected_registers.set(PC, 0x0101);
                let expected_memory = memory.clone();

                let cycles = process_instruction(&mut registers, &mut memory);
                assert_eq!(4, cycles);
                assert_eq!(expected_registers, registers);
                assert_eq!(expected_memory, memory);
            }
        }
    }

    macro_rules! load_test_to_memory {
        ($instruction: tt, $to: tt, $from: tt) => {
            {
                let mut registers = Registers::new();
                registers.set($from, 0xff);
                registers.set($to, 0xD000);
                let mut memory = Memory::init_empty_with_instruction(0x0100, &[$instruction]);

                let mut expected_registers = registers.clone();
                expected_registers.set(PC, 0x0101);
                let mut expected_memory = memory.clone();
                expected_memory.write(0xD000, 0xFF);

                let cycles = process_instruction(&mut registers, &mut memory);
                assert_eq!(8, cycles);
                assert_eq!(expected_registers, registers);
                assert_eq!(expected_memory, memory);
            }
        }
    }

    macro_rules! dec_test {
        ($instruction: tt, $target: tt, $expected_value: tt, $init_value: tt,
         $expected_flags: tt, $init_flags: tt, $expected_cycles: tt) => {
            {
                let mut registers = Registers::new();
                registers.set($target, $init_value);
                registers.set(F, $init_flags);
                let mut memory = Memory::init_empty_with_instruction(0x0100, &[$instruction]);

                let mut expected_registers = registers.clone();
                expected_registers.set($target, $expected_value);
                expected_registers.set(PC, 0x0101);
                expected_registers.set(F, $expected_flags);
                let expected_memory = memory.clone();

                let cycles = process_instruction(&mut registers, &mut memory);
                assert_eq!($expected_cycles, cycles);
                assert_eq!(expected_registers, registers);
                assert_eq!(expected_memory, memory);
            }
        }
    }

    macro_rules! dec_test_memory {
        ($instruction: tt, $target: tt, $expected_value: tt, $init_value: tt,
         $expected_flags: tt, $init_flags: tt, $expected_cycles: tt) => {
            {
                let mut registers = Registers::new();
                registers.set($target, 0xD000);
                registers.set(F, $init_flags);
                let mut memory = Memory::init_empty_with_instruction(0x0100, &[$instruction]);
                memory.write(0xD000, $init_value);

                let mut expected_registers = registers.clone();
                expected_registers.set(PC, 0x0101);
                expected_registers.set(F, $expected_flags);
                let mut expected_memory = memory.clone();
                expected_memory.write(0xD000, $expected_value);

                let cycles = process_instruction(&mut registers, &mut memory);
                assert_eq!(expected_registers, registers);
                assert_eq!(expected_memory, memory);
                assert_eq!($expected_cycles, cycles);
            }
        }
    }

    #[test]
    fn test_0x05() {
        dec_test!(0x05, B, 0b11111111, 0b00000000, 0b01000000, 0x0, 4);
        dec_test!(0x05, B, 0b00100000, 0b00100001, 0b01100000, 0x0, 4);
        dec_test!(0x05, B, 0b00001111, 0b00010000, 0b01000000, 0x0, 4);
        dec_test!(0x05, B, 0b00000000, 0b00000001, 0b11100000, 0x0, 4);
    }

    #[test]
    fn test_0x15() {
        dec_test!(0x15, D, 0b11111111, 0b00000000, 0b01000000, 0x0, 4);
        dec_test!(0x15, D, 0b00100000, 0b00100001, 0b01100000, 0x0, 4);
        dec_test!(0x15, D, 0b00001111, 0b00010000, 0b01000000, 0x0, 4);
        dec_test!(0x15, D, 0b00000000, 0b00000001, 0b11100000, 0x0, 4);
    }

    #[test]
    fn test_0x25() {
        dec_test!(0x25, H, 0b11111111, 0b00000000, 0b01000000, 0x0, 4);
        dec_test!(0x25, H, 0b00100000, 0b00100001, 0b01100000, 0x0, 4);
        dec_test!(0x25, H, 0b00001111, 0b00010000, 0b01000000, 0x0, 4);
        dec_test!(0x25, H, 0b00000000, 0b00000001, 0b11100000, 0x0, 4);
    }

    #[test]
    fn test_0x35() {
        dec_test_memory!(0x35, HL, 0b11111111, 0b00000000, 0b01000000, 0x0, 12);
        dec_test_memory!(0x35, HL, 0b00100000, 0b00100001, 0b01100000, 0x0, 12);
        dec_test_memory!(0x35, HL, 0b00001111, 0b00010000, 0b01000000, 0x0, 12);
        dec_test_memory!(0x35, HL, 0b00000000, 0b00000001, 0b11100000, 0x0, 12);
    }

    #[test]
    fn test_0x0B() {
        dec_test!(0x0B, BC, 0xFFFF, 0x0000, 0x00, 0x00, 8)
    }

    #[test]
    fn test_0x1B() {
        dec_test!(0x1B, DE, 0xFFFF, 0x0000, 0x00, 0x00, 8)
    }

    #[test]
    fn test_0x2B() {
        dec_test!(0x2B, HL, 0xFFFF, 0x0000, 0x00, 0x00, 8)
    }

    #[test]
    fn test_0x3B() {
        dec_test!(0x3B, SP, 0xFFFF, 0x0000, 0x00, 0x00, 8)
    }

    #[test]
    fn test_0x2F() {
        let mut registers = Registers::new();
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0x2F]);

        let mut expected_registers = registers.clone();
        expected_registers.set(PC, 0x0101);
        expected_registers.set(F, 0b11110000);
        expected_registers.set(A, 0xFE);
        let expected_memory = memory.clone();

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(4, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }

    #[test]
    fn test_0xC3() {
        let mut registers = Registers::new();
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0xC3, 0x12, 0x34]);

        let mut expected_registers = registers.clone();
        expected_registers.set(PC, 0x1234);
        let expected_memory = memory.clone();

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(16, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }

    #[test]
    fn test_0x00() {
        let mut registers = Registers::new();
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0x00]);

        let mut expected_registers = registers.clone();
        expected_registers.set(PC, 0x0101);
        let expected_memory = memory.clone();

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(4, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }

    #[test]
    fn test_x42() {
        load_test_only_reg!(0x42, B, D);
    }

    #[test]
    fn test_x52() {
        load_test_only_reg!(0x52, D, D);
    }

    #[test]
    fn test_x62() {
        load_test_only_reg!(0x62, H, D);
    }

    #[test]
    fn test_x72() {
        load_test_to_memory!(0x72, HL, D);
    }

    #[test]
    fn test_x4A() {
        load_test_only_reg!(0x4A, C, D);
    }

    #[test]
    fn test_x5A() {
        load_test_only_reg!(0x5A, E, D);
    }

    #[test]
    fn test_x6A() {
        load_test_only_reg!(0x6A, L, D);
    }

    #[test]
    fn test_x7A() {
        load_test_only_reg!(0x7A, A, D);
    }

    #[test]
    fn test_x40() {
        load_test_only_reg!(0x40, B, B);
    }

    #[test]
    fn test_x50() {
        load_test_only_reg!(0x50, D, B);
    }

    #[test]
    fn test_x60() {
        load_test_only_reg!(0x60, H, B);
    }

    #[test]
    fn test_x70() {
        load_test_to_memory!(0x70, HL, B);
    }

    #[test]
    fn test_x48() {
        load_test_only_reg!(0x48, C, B);
    }

    #[test]
    fn test_x58() {
        load_test_only_reg!(0x58, E, B);
    }

    #[test]
    fn test_x68() {
        load_test_only_reg!(0x68, L, B);
    }

    #[test]
    fn test_x78() {
        load_test_only_reg!(0x78, A, B);
    }

    #[test]
    fn test_x41() {
        load_test_only_reg!(0x41, B, C);
    }

    #[test]
    fn test_x51() {
        load_test_only_reg!(0x51, D, C);
    }

    #[test]
    fn test_x61() {
        load_test_only_reg!(0x61, H, C);
    }

    #[test]
    fn test_x71() {
        load_test_to_memory!(0x71, HL, C)
    }

    #[test]
    fn test_x49() {
        load_test_only_reg!(0x49, C, C);
    }

    #[test]
    fn test_x59() {
        load_test_only_reg!(0x59, E, C);
    }

    #[test]
    fn test_x69() {
        load_test_only_reg!(0x69, L, C);
    }

    #[test]
    fn test_x79() {
        load_test_only_reg!(0x79, A, C);
    }
}