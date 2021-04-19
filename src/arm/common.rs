pub struct HalfWord {
    pub bytes: [u8; 2]
}

impl HalfWord {
    pub fn little_endian(&self) -> u16 {
        (self.bytes[1] as u16) << 8 | (self.bytes[0] as u16)
    }

    pub fn big_endian(&self) -> u16 {
        (self.bytes[0] as u16) << 8 | (self.bytes[1] as u16)
    }
}

pub struct Word {
    pub bytes: [u8; 4]
}

impl Word {
    pub fn little_endian(&self) -> u32 {
        (self.bytes[3] as u32) << 24 | (self.bytes[2] as u32) << 16 | (self.bytes[1] as u32) << 8 | (self.bytes[0] as u32)
    }

    pub fn big_endian(&self) -> u32 {
        (self.bytes[0] as u32) << 24 | (self.bytes[1] as u32) << 16 | (self.bytes[2] as u32) << 8 | (self.bytes[3] as u32)
    }
}