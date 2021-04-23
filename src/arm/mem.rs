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
    pub fn set_byte(&mut self, byte_index: usize, data: u8) {
        self.mem[byte_index] = data;
    }

    #[inline(always)]
    pub fn set_halfword(&mut self, byte_index: usize, data: HalfWord) {
        self.mem[byte_index] = data.bytes[0];
        self.mem[byte_index + 1] = data.bytes[1];
    }

    #[inline(always)]
    pub fn set_word(&mut self, byte_index: usize, data: Word) {
        self.mem[byte_index] = data.bytes[0];
        self.mem[byte_index + 1] = data.bytes[1];
        self.mem[byte_index + 2] = data.bytes[2];
        self.mem[byte_index + 3] = data.bytes[3];
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