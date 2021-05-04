pub mod arm;
pub mod audio;
pub mod graphics;
extern crate gio;

use gtk::{Image, prelude::*};
use gio::prelude::*;

use std::env;
use std::fs::File;
use std::fs;
use std::io::Read;
use std::time::{Duration, Instant};
use graphics::gpu::{draw};
use show_image::{ImageView, ImageInfo, create_window};
use anyhow::Result;
use arm::{cpu, mem};
use arm::mem::Mem;
use arm::cpu::Cpu;
use audio::apu::make_a_sound;
use gtk::{Application, ApplicationWindow, Button};


fn main() -> Result<()> {
    let application = Application::new(
        Some("com.github.gtk-rs.examples.basic"),
        Default::default(),
    ).expect("failed to initialize GTK application");

    application.connect_activate(|app| {
        let window = ApplicationWindow::new(app);
        window.set_title("First GTK Program");
        window.set_default_size(960, 640);
        let mut cpu = Cpu::new();
        let mut f = File::open("memdump.txt").expect("no file found");
        let mut mem = Mem::new(235_000_000);
        mem.load(0x0, f);
        /*for x in 0x06000000..0x06017FFF{
            print!("{}", memory[x]);
        }*/
    
        let elapsed = Duration::from_nanos(69);
        let image = Image::from_pixbuf(Some(&draw(&mut mem, &mut cpu, elapsed)));
        window.add(&image);
        window.show_all();
    });

    application.run(&[]);
    /* 
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
    
    c
    let gpuCycleStart = Instant::now();
    let mut instructions = 0;
    while instructions < 800_000 {
        instructions += 1;
        if cpu.step(&mut ram).is_none() {
            break;
        }
        elapsed = Instant::now().duration_since(gpuCycleStart);
        window.set_image("RBA", draw(&mut ram, & mut cpu, elapsed));
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
     */
    Ok(())
}
