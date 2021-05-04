pub mod arm;
pub mod audio;
pub mod graphics;

use std::{env, thread::sleep};
use std::fs::File;
use std::time::{Duration, Instant};
use graphics::gpu::{draw};
use show_image::{ImageView, ImageInfo, create_window};
use anyhow::Result;
use arm::{cpu, mem};
use audio::apu::make_a_sound;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut ram = mem::Mem::new(235_000_000);
    println!("Loading memory...");
    ram.load(0, File::open(&args[1])?)?;
    ram.load(0x8000000, File::open(&args[2])?)?;
    println!("Starting simulation.");
    let mut cpu = cpu::Cpu::new();
    cpu.reset();
    //cpu.toggle_debug();
    let window = create_window("RBA", Default::default()).unwrap();
    cpu.toggle_debug();
    let two_clock_cycles = Duration::from_nanos(5);
    let gpu_cycle_start = Instant::now();
    
    let mut cycles = 0;
    while cycles < 10_000_000 {
        if cpu.step(&mut ram).is_none() {
            break;
        }
        draw(&mut ram, cycles);
        //window.set_image("RBA", draw(&mut ram, cycles));
        cycles += 2;
    }
    println!("Saving state.");
    let file = File::create("logs/wram_dump.hex").unwrap();
    ram.save(0x3000000, 0x3007FFF, file).unwrap();
    let file = File::create("logs/palette_dump.hex").unwrap();
    ram.save(0x5000000, 0x50003FF, file).unwrap();
    let file = File::create("logs/vram_dump.hex").unwrap();
    ram.save(0x6000000, 0x6017FFF, file).unwrap();
    let file = File::create("logs/oam_dump.hex").unwrap();
    ram.save(0x7000000, 0x70003FF, file).unwrap();
    let file = File::create("logs/graphics_dump.hex").unwrap();
    ram.save(0x4000000, 0x70003FF, file).unwrap();

    println!("Took {} ms", Instant::now().duration_since(gpu_cycle_start).as_millis());
    // For zlib test
    // let mut i = 40000;
    // loop {
    //     if ram.get_byte(i) == 0 {break;}
    //     print!("{}", ram.get_byte(i) as char);
    //     i += 1;
    // }
    // println!();

    // createDisplay();
    //make_a_sound();
    println!("{} cycles", cycles);
    Ok(())
}
