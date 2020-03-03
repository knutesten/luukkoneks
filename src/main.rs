mod register;
mod memory;
mod opcode;
mod lcd;

fn main() {
    let mut registers = register::Registers::new();
    let mut memory = memory::Memory::init_from_rom("Tetris.gb").unwrap();

    loop {
        opcode::process_instruction(&mut registers, &mut memory);
    }
}
