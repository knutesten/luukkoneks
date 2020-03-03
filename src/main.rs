mod register;
mod memory;
mod opcode;

fn main() {
    let registers = register::Registers::new();
    println!("{:#?}", registers);
    let memory = memory::Memory::init_from_rom("Tetris.gb").expect("Shit ass");
    println!("Loaded shit!")
}
