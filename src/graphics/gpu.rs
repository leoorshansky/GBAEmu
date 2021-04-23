use crate::arm::mem::Mem;

const PRAM_START: u64 = 0x05000000;
const PRAM_END: u64 = 0x050003FF;
const VRAM_START: u64 = 0x06000000;
const VRAM_END: u64 = 0x06017FFF;
const OAM_START: u64 = 0x07000000;
const OAM_END: u64 = 0x070003FF;