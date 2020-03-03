use std::thread::sleep;
use std::time::Duration;

use crate::memory::Memory;
use crate::register::Registers;
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
        _ => panic!("Unsupported load instruction {}", instruction)
    };

    let value = registers.get(from);
    registers.set(to, value);

    4
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_x40() {
        let mut registers = Registers::new();
        let mut memory = Memory::init_empty_with_instruction(0x0100, &[0x40]);

        let expected_registers = registers.clone();
        let expected_memory = memory.clone();

        let cycles = process_instruction(&mut registers, &mut memory);
        assert_eq!(4, cycles);
        assert_eq!(expected_registers, registers);
        assert_eq!(expected_memory, memory);
    }
}