pub mod arm;
pub mod graphics;
pub mod audio;

use std::fs::File;

use arm::{cpu, mem};
use anyhow::Result;
use graphics::window::createDisplay;
use audio::apu::make_a_sound;

fn main() -> Result<()> {
    let mut ram = mem::Mem::new(300_000_000);
    //ram.load(0, File::open("gba_bios.bin")?)?;
    //ram.load(0, File::open("tests/test_ldrh.bin")?)?;
    let mut cpu = cpu::Cpu::new();
    cpu.reset();
    cpu.toggle_debug();
    /*
    for _ in 0..200 {
        cpu.step(&mut ram);
    }
    */
    // createDisplay();
    //make_a_sound();
    Ok(())
}
