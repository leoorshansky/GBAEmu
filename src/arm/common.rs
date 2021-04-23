pub struct HalfWord {
    pub bytes: [u8; 2]
}

impl HalfWord {
    #[inline(always)]
    pub fn little_endian(&self) -> u16 {
        u16::from_le_bytes(self.bytes)
    }

    #[inline(always)]
    pub fn big_endian(&self) -> u16 {
        u16::from_be_bytes(self.bytes)
    }

    #[inline(always)]
    pub fn from_u16_le(data: u16) -> Self {
        Self {
            bytes: data.to_le_bytes()
        }
    }

    #[inline(always)]
    pub fn from_u16_be(data: u16) -> Self {
        Self {
            bytes: data.to_be_bytes()
        }
    }
}

pub struct Word {
    pub bytes: [u8; 4]
}

impl Word {
    #[inline(always)]
    pub fn little_endian(&self) -> u32 {
        u32::from_le_bytes(self.bytes)
    }

    #[inline(always)]
    pub fn big_endian(&self) -> u32 {
        u32::from_be_bytes(self.bytes)
    }

    #[inline(always)]
    pub fn from_u32_le(data: u32) -> Self {
        Self {
            bytes: data.to_le_bytes()
        }
    }

    #[inline(always)]
    pub fn from_u32_be(data: u32) -> Self {
        Self {
            bytes: data.to_be_bytes()
        }
    }
}