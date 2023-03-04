use clap::Parser;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

mod cpu;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    rom_path: String,
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    
    if args.rom_path.is_empty() {
        return Ok(());
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

    println!("Opening window....");

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Chip-8 Rust Emulator", 768, 384)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    println!("window is now opened....");

    let mut event_pump = sdl_context.event_pump()?;
    
    'running: loop {
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        cpu.step();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for i in 0..64 {
            for j in 0..32 {
                if cpu.vram[i][j] > 0 {
                    canvas.fill_rect(Rect::new(i as i32 * 12, j as i32 * 12, 12, 12))?;
                }
            }
        }
        canvas.present();
    }

    // loop {
    //     cpu.step();
    // }

    Ok(())
}
