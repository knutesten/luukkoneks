mod register;
mod memory;

fn main() {
    let registers = register::Registers::new();

    println!("{:#?}", registers);

    let memory = memory::Memory::init_from_rom("Tetris.gb").expect("Shit ass");

    println!("Loaded shit!")
}
