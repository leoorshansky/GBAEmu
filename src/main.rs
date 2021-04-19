pub mod arm;

use std::fs::File;

use arm::mem;
use anyhow::Result;

fn main() -> Result<()> {
    let mut ram = mem::Mem::new(300_000_000);
    ram.load(0, File::open("gba_bios.bin")?)?;
    
    Ok(())
}
