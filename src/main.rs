pub mod arm;
pub mod audio;
pub mod graphics;

use std::env;
use std::fs::File;
use std::time::{Duration, Instant};
//use graphics::gpu::{draw};
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
    let mut elapsed = Duration::from_millis(0);
    let gpuCycleStart = Instant::now();
    let mut instructions = 0;
    while instructions < 800_000 {
        instructions += 1;
        if cpu.step(&mut ram).is_none() {
            break;
        }
        elapsed += Instant::now().duration_since(gpuCycleStart);
        //draw(&mut ram, elapsed);
    }

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
    println!("{}", instructions);
    Ok(())
}
