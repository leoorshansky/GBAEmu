pub mod arm;
pub mod audio;
pub mod graphics;

use std::{env, thread::sleep};
use std::fs::File;
use std::fs;
use std::io::Read;
use std::time::{Duration, Instant};
use graphics::gpu::{draw};
use arm::{cpu, mem};
use arm::cpu::Cpu;
use audio::apu::APU;

fn main() {
    let args: Vec<String> = env::args().collect();

    //let mut f = File::open("memdump.txt").expect("no file found");

    // let sdl_context = sdl2::init().unwrap();
    // let video_subsystem = sdl_context.video().unwrap();

    // let window = video_subsystem.window("rust-sdl2 demo", 960, 640)
    //     .position_centered()
    //     .build()
    //     .unwrap();

    // let mut canvas = window.into_canvas().build().unwrap();
    // canvas.clear();
    // canvas.present();

    let mut cpu = Cpu::new();
    let mut ram = mem::Mem::new(235_000_000);
    println!("Loading memory...");
    ram.load(0, File::open(&args[1]).unwrap()).unwrap();
    //ram.load(0x8000000, File::open(&args[2]).unwrap()).unwrap();
    /*for x in 0x06000000..0x06017FFF{
        print!("{}", memory[x]);
    }*/
    println!("Starting simulation.");
    cpu.reset();
    //cpu.toggle_debug();
    let two_clock_cycles = Duration::from_nanos(5);
    let gpu_cycle_start = Instant::now();
    // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // let mut apu = APU::new(&stream_handle);
    let mut cycles = 0;
    while cycles < 100_000_000 {
        if cpu.step(&mut ram, cycles).is_none() {
            break;
        }
        // if cycles % 100 == 0 {
        //     draw(&mut ram, cycles, &mut canvas);
        // }
        //apu.step(&ram);
        cycles += 2;
    }

    println!("Took {} ms", Instant::now().duration_since(gpu_cycle_start).as_millis());
    println!("Ran {} cycles", cycles);

    // println!("Saving state.");
    // let file = File::create("logs/wram_dump.hex").unwrap();
    // ram.save(0x3000000, 0x3007FFF, file).unwrap();
    // let file = File::create("logs/palette_dump.hex").unwrap();
    // ram.save(0x5000000, 0x50003FF, file).unwrap();
    // let file = File::create("logs/vram_dump.hex").unwrap();
    // ram.save(0x6000000, 0x6017FFF, file).unwrap();
    // let file = File::create("logs/oam_dump.hex").unwrap();
    // ram.save(0x7000000, 0x70003FF, file).unwrap();
    // let file = File::create("logs/graphics_dump.hex").unwrap();
    // ram.save(0x4000000, 0x70003FF, file).unwrap();
    
    let mut i = 40000;
    loop {
        if ram.get_byte(i) == 0 {break;}
        print!("{}", ram.get_byte(i) as char);
        i += 1;
    }
    println!();

    // createDisplay();
    //make_a_sound();
    Ok(())
}
