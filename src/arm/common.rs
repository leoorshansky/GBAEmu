pub struct HalfWord {
    pub bytes: [u8; 2]
}

impl HalfWord {
    pub fn little_endian(&self) -> u16 {
        u16::from_le_bytes(self.bytes)
    }

    pub fn big_endian(&self) -> u16 {
        u16::from_be_bytes(self.bytes)
    }
}

pub struct Word {
    pub bytes: [u8; 4]
}

impl Word {
    pub fn little_endian(&self) -> u32 {
        u32::from_le_bytes(self.bytes)
    }

    pub fn big_endian(&self) -> u32 {
        u32::from_be_bytes(self.bytes)
    }
}