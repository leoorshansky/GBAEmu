use std::io::Read;
use anyhow::Result;

use crate::arm::common::{HalfWord, Word};

pub struct Mem {
    mem: Vec<u8>
}

impl Mem {
    pub fn new(size: usize) -> Self {
        let mut vec = Vec::with_capacity(size);
        unsafe { vec.set_len(size); }
        Mem {
            mem: vec
        }
    }

    pub fn load(&mut self, offset: usize, bios: impl Read) -> Result<()> {
        for (i, byte) in bios.bytes().enumerate() {
            self.mem[offset + i] = byte?;
        }
        Ok(())
    }

    #[inline(always)]
    pub fn get_byte(&self, byte_index: usize) -> u8 {
        self.mem[byte_index]
    }

    #[inline(always)]
    pub fn get_halfword(&self, byte_index: usize) -> HalfWord {
        HalfWord {
            bytes: [self.mem[byte_index], self.mem[byte_index + 1]]
        }
    }

    #[inline(always)]
    pub fn get_word(&self, byte_index: usize) -> Word {
        Word {
            bytes: [self.mem[byte_index], self.mem[byte_index + 1], self.mem[byte_index + 2], self.mem[byte_index + 3]]
        }
    }
}