use std::{time::Duration, u8, usize};

use crate::arm::mem::Mem;
use crate::arm::common::{HalfWord};

const PRAM_START: usize = 0x05000000;
const PRAM_END: usize = 0x050003FF;
const VRAM_START: usize = 0x06000000;
const VRAM_END: usize = 0x06017FFF;
const OAM_START: usize = 0x07000000;
const OAM_END: usize = 0x070003FF;

//Control Register Information
const REG_DISPCNT_ADDR: usize = 0x00400000;
const VideoMode_START_BIT: u8 = 0;
const VideoMode_END_BIT: u8 = 2;
const GameBoy_BIT: u8 = 3;
const PageSelect_BIT: u8 = 4;
const OAM_HBlank_BIT: u8 = 5;
const ObjectMappingMode_BIT: u8 = 6;
const ForceBlank_BIT: u8 = 7;
const BG0Display_BIT: u8 = 8;
const BG1Display_BIT: u8 = 9;
const BG2Display_BIT: u8 = 10;
const BG3Display_BIT: u8 = 11;
const OBJDisplay_BIT: u8 = 12;
const Window0Display_BIT: u8 = 13;
const Window1Display_BIT: u8 = 14;
const OBJWindowDisplay_BIT: u8 = 15;
//Display Status Register Information
const REG_DISPSTAT_ADDR: usize = 0x00400004;
const VBlank_BIT: u8 = 0;
const HBlank_BIT: u8 = 1;
const VCountTrigger_BIT: u8 = 2;
const VBlankInterruptRequest_BIT: u8 = 3;
const HBlankInterruptRequest_BIT: u8 = 4;
const VCountInterruptRequest_BIT: u8 = 5;
const VCountTriggerValue_START_BIT: u8 = 8;
const VCountTriggerValue_END_BIT: u8 = 15;
//VCounter Register Information
const REG_VCOUNT_ADDR: usize = 0x00400006;

//Timing constants
const CYCLE_TIME: usize = 16666666;
const V_BLANK_TIME: usize = 11695906;

struct Register{
    value: u16,
    address: usize,
}

struct Pixel{
    R: u8,
    G: u8,
    B: u8
}

impl Register{
    fn getValue(&self) -> u16{
       self.value
    }
    fn setValue(&mut self, v: u16, mem: &mut Mem) -> (){
        self.value = v;
        let hw = HalfWord::from_u16_le(self.value);
        mem.set_halfword(self.address, hw);        
    }
    fn setBit(&mut self, v: u8, n: u8, mem: &mut Mem) -> (){
        if v == 0 {
            let i: u16 = 2^16 - 2^(n as u16);
            self.value = self.value & i;
        }
        else if v == 1 {
            let i: u16 = 2^(n as u16);
            self.value = self.value | i;
        }
        else{
            //this should never occur
            println!("your mom");
        }
        let hw = HalfWord::from_u16_le(self.value);
        mem.set_halfword(self.address, hw);   
    }
}


pub fn draw(mem: &mut Mem, elapsed: Duration) -> (){
    let mut control = Register {
        value: mem.get_halfword(REG_DISPCNT_ADDR).little_endian(),
        address: REG_DISPCNT_ADDR,
    };
    let mut status = Register {
        value: mem.get_halfword(REG_DISPSTAT_ADDR).little_endian(),
        address: REG_DISPSTAT_ADDR,
    };
    let mut vCounter = Register {
        value: mem.get_halfword(REG_VCOUNT_ADDR).little_endian(),
        address: REG_VCOUNT_ADDR,
    };
    let mut currentCycle = elapsed.as_nanos() % CYCLE_TIME as u128;
    if(currentCycle > V_BLANK_TIME as u128){
        status.setBit(1, 0, mem);
    }
    else{
        status.setBit(0, 0, mem);
    }

}