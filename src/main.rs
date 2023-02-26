use std::fs::File;
use std::fs::read;

mod cpu;

fn main() {
    let mut cpu: cpu::Cpu = cpu::Cpu::new();

    let path = "/home/todd/Downloads/ibm_logo.ch8";

    println!("Loading rom.....");
    let rom = if let Ok(bytes_read) = std::fs::read(path) {
	bytes_read
    } else {
	panic!("unable to read the provided rom....");
    };

    cpu.load_rom(rom);

    println!("rom is loaded....");

    loop {
	cpu.step();
    }
    
}
