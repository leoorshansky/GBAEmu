use anyhow::Result;
use std::io::{ErrorKind, Read, Write};

use crate::arm::common::{HalfWord, Word};

pub struct Mem {
    mem: Vec<u8>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Interrupt {
    VBlank,
    HBlank,
    VCounter,
    Timer0,
    Timer1,
    Timer2,
    Timer3,
    Serial,
    Dma0,
    Dma1,
    Dma2,
    Dma3,
    Keypad,
    GamePak
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Key {
    A,
    B,
    Select,
    Start,
    Right,
    Left,
    Up,
    Down,
    R,
    L
}

impl Mem {
    pub fn new(size: usize) -> Self {
        let mut vec = Vec::with_capacity(size);
        unsafe {
            vec.set_len(size);
        }
        Mem { mem: vec }
    }

    pub fn load(&mut self, first_byte: usize, mut file: impl Read) -> std::io::Result<()> {
        let buf = &mut self.mem[first_byte..];
        if let Err(e) = file.read_exact(buf) {
            if e.kind() != ErrorKind::UnexpectedEof {
                return Err(e);
            }
        }   
        Ok(())
    }

    pub fn save(&self, first_byte: usize, last_byte: usize, mut file: impl Write) -> std::io::Result<()> {
        let buf = &self.mem[first_byte..=last_byte];
        file.write_all(buf)
    }

    #[inline(always)]
    pub fn set_byte(&mut self, mut byte_index: usize, data: u8) {
        if byte_index == 0x4000202 {
            self.mem[byte_index] &= !data;
            return;
        }
        if (0x3FFFF00..=0x3FFFFFF).contains(&byte_index) {
            byte_index -= 0xFF8000;
        } else if (0xA000000..=0xBFFFFFF).contains(&byte_index) {
            byte_index -= 0x2000000;
        } else if (0xC000000..=0xDFFFFFF).contains(&byte_index) {
            byte_index -= 0x4000000;
        }
        self.mem[byte_index] = data;
    }

    #[inline(always)]
    pub fn set_halfword(&mut self, mut byte_index: usize, data: HalfWord) {
        if byte_index == 0x4000202 {
            self.mem[byte_index] &= !data.bytes[0];
            self.mem[byte_index + 1] &= !data.bytes[1];
            return;
        }
        if (0x3FFFF00..=0x3FFFFFF).contains(&byte_index) {
            byte_index -= 0xFF8000;
        } else if (0xA000000..=0xBFFFFFF).contains(&byte_index) {
            byte_index -= 0x2000000;
        } else if (0xC000000..=0xDFFFFFF).contains(&byte_index) {
            byte_index -= 0x4000000;
        }
        self.mem[byte_index] = data.bytes[0];
        self.mem[byte_index + 1] = data.bytes[1];
    }

    #[inline(always)]
    pub fn set_word(&mut self, mut byte_index: usize, data: Word) {
        if byte_index == 0x4000202 {
            self.mem[byte_index] &= !data.bytes[0];
            self.mem[byte_index + 1] &= !data.bytes[1];
            self.mem[byte_index + 2] = data.bytes[2];
            self.mem[byte_index + 3] = data.bytes[3];
            return;
        }
        if (0x3FFFF00..=0x3FFFFFF).contains(&byte_index) {
            byte_index -= 0xFF8000;
        } else if (0xA000000..=0xBFFFFFF).contains(&byte_index) {
            byte_index -= 0x2000000;
        } else if (0xC000000..=0xDFFFFFF).contains(&byte_index) {
            byte_index -= 0x4000000;
        }
        self.mem[byte_index] = data.bytes[0];
        self.mem[byte_index + 1] = data.bytes[1];
        self.mem[byte_index + 2] = data.bytes[2];
        self.mem[byte_index + 3] = data.bytes[3];
    }

    #[inline(always)]
    pub fn get_byte(&self, mut byte_index: usize) -> u8 {
        if (0x3FFFF00..=0x3FFFFFF).contains(&byte_index) {
            byte_index -= 0xFF8000;
        } else if (0xA000000..=0xBFFFFFF).contains(&byte_index) {
            byte_index -= 0x2000000;
        } else if (0xC000000..=0xDFFFFFF).contains(&byte_index) {
            byte_index -= 0x4000000;
        }
        self.mem[byte_index]
    }

    #[inline(always)]
    pub fn get_halfword(&self, mut byte_index: usize) -> HalfWord {
        if (0x3FFFF00..=0x3FFFFFF).contains(&byte_index) {
            byte_index -= 0xFF8000;
        } else if (0xA000000..=0xBFFFFFF).contains(&byte_index) {
            byte_index -= 0x2000000;
        } else if (0xC000000..=0xDFFFFFF).contains(&byte_index) {
            byte_index -= 0x4000000;
        }
        HalfWord {
            bytes: [self.mem[byte_index], self.mem[byte_index + 1]],
        }
    }

    #[inline(always)]
    pub fn get_word(&self, mut byte_index: usize) -> Word {
        if (0x3FFFF00..=0x3FFFFFF).contains(&byte_index) {
            byte_index -= 0xFF8000;
        } else if (0xA000000..=0xBFFFFFF).contains(&byte_index) {
            byte_index -= 0x2000000;
        } else if (0xC000000..=0xDFFFFFF).contains(&byte_index) {
            byte_index -= 0x4000000;
        }
        Word {
            bytes: [
                self.mem[byte_index],
                self.mem[byte_index + 1],
                self.mem[byte_index + 2],
                self.mem[byte_index + 3],
            ],
        }
    }

    pub fn key_event(&mut self, key: Key, down: bool) {
        let bit = match key {
            Key::A => 0,
            Key::B => 1,
            Key::Select => 2,
            Key::Start => 3,
            Key::Right => 4,
            Key::Left => 5,
            Key::Up => 6,
            Key::Down => 7,
            Key::R => 8,
            Key::L => 9
        };
        let mask = 1 << bit;
        let new_key_inputs = (self.get_halfword(0x4000130).little_endian() & !mask) | (!down as u16) << bit;
        self.set_halfword(0x4000130, HalfWord::from_u16_le(new_key_inputs));

        let key_ctl = self.get_halfword(0x4000132).little_endian();
        if down && key_ctl >> 14 & 1 == 1 {
            if key_ctl >> 15 & 1 == 1 { // AND
                if !new_key_inputs & key_ctl == key_ctl {
                    self.request_irq(Interrupt::Keypad);
                }
            } else { // OR
                if key_ctl >> bit & 1 == 1 {
                    self.request_irq(Interrupt::Keypad);
                }
            }
        }
    }

    pub fn request_irq(&mut self, kind: Interrupt) {
        if self.mem[0x4000208] & 1 == 0 { // IME
            return;
        }
        let bit = match kind {
            Interrupt::VBlank => 0,
            Interrupt::HBlank => 1,
            Interrupt::VCounter => 2,
            Interrupt::Timer0 => 3,
            Interrupt::Timer1 => 4,
            Interrupt::Timer2 => 5,
            Interrupt::Timer3 => 6,
            Interrupt::Serial => 7,
            Interrupt::Dma0 => 8,
            Interrupt::Dma1 => 9,
            Interrupt::Dma2 => 10,
            Interrupt::Dma3 => 11,
            Interrupt::Keypad => 12,
            Interrupt::GamePak => 13
        };
        if self.mem[0x4000200] >> bit & 1 == 1 { // IE
            if bit < 8 {
                self.mem[0x4000202] |= 1 << bit;
            } else {
                self.mem[0x4000203] |= 1 << (bit - 8);
            }
        }
    }
}
