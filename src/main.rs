mod register;

fn main() {
    let registers = register::Registers::new();

    println!("{:#?}", registers);
}
