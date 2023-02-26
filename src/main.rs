use clap::Parser;
use std::fs::read;
use std::fs::File;

mod cpu;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    rom_path: String,
}

fn main() {
    let args = Args::parse();

    if args.rom_path.is_empty() {
        return;
    }

    let mut cpu: cpu::Cpu = cpu::Cpu::new();

    println!("Loading rom.....");
    let rom = if let Ok(bytes_read) = std::fs::read(args.rom_path.as_str()) {
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
