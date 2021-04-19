pub mod arm;

use std::fs::File;

use arm::{cpu, mem};
use anyhow::Result;

fn main() -> Result<()> {
    let mut ram = mem::Mem::new(300_000_000);
    //ram.load(0, File::open("gba_bios.bin")?)?;
    ram.load(0, File::open("test_alu.bin")?)?;
    let mut cpu = cpu::Cpu::new(ram);
    cpu.reset();
    cpu.toggle_debug();
    for _ in 0..100 {
        cpu.step();
    }
    Ok(())
}
