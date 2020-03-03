use crate::opcode::process_instruction;

mod register;
mod memory;
mod opcode;
mod lcd;

fn main() {
    let mut registers = register::Registers::new();
    let mut memory = memory::Memory::init_from_rom("Tetris.gb").expect("Shit ass");

    loop {
        process_instruction(&mut registers, &mut memory);
    }
}
