use std::fs::File;
use std::io::Read;
mod emulator;


fn main() {
    let mut instruction_file = File::open("./src/instruction_sets/pong.rom").unwrap();
    let mut instruction_set = Vec::<u8>::new();

    instruction_file.read_to_end(&mut instruction_set);
    let hi = instruction_set[40] as u16;
    let lo = instruction_set[41] as u16;
    println!("{:#b}", hi);
    println!("{:#b}", lo);

    println!("{:#b}", hi << 8);

    let instruction = (hi << 8) | lo;
    println!("{:#x}", instruction);

    println!("{:#x}", (instruction & 0xF000));
    println!("{:#x}", (instruction & 0xF000) >> 12);
}
