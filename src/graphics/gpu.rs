use std::{time::Duration, u16, u8, usize};
use image::{RgbImage, Rgb};

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
//BG Control Register Information
const BG_CNTRL_ADDR: [usize; 4] = [0x04000008, 0x0400000A, 0x0400000C, 0x0400000E];
const BG_PRIORITY_START_BIT: u8 = 0;
const BG_PRIORITY_END_BIT: u8 = 1;
const CHARACTER_BASE_START_BIT: u8 = 2;
const CHARACTER_BASE_END_BIT: u8 = 3;
const MOSAIC_BIT: u8 = 6;
const PALETTES_BIT: u8 = 7;
const SCREEN_BASE_START_BIT: u8 = 8;
const SCREEN_BASE_END_BIT: u8 = 12;
const SCREEN_SIZE_START_BIT: u8 = 14;
const SCREEN_SIZE_END_BIT: u8 = 15;
//I/O BG Offset Registers
const BG_HORIZONTAL_OFFSET_ADDR: [usize; 4] = [0x4000010, 0x4000014, 0x4000018, 0x400001C];
const BG_VERTICAL_OFFSET_ADDR: [usize; 4] = [0x4000012, 0x4000016, 0x400001A, 0x400001E];
//Timing constants
const CYCLE_TIME: usize = 16666666;
const V_BLANK_TIME: usize = 11695906;
const SCANLINE_TIME: usize = 73099;
const H_BLANK_TIME: usize = 56960;
//Tile Data Information
const TILE_DATA_ADDR:usize = 0x06000000;
//Bitmap addresses
const BITMAP_DATA_ADDR : usize = 0x06000000;
const MODE3_DATA_ENDADDR : usize = 0x06013FFF;
const MODE45_DATA_ENDADDR : usize = 0x06009FFF;
//OBJ Tile Addresses 
const OBJ_DATA_ADDR : usize = 0x06014000;
const OBJ_DATA_ENDADDR : usize = 0x06017FFF


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
    fn getBit(&self, n: u16) -> u16{
        let hw = HalfWord::from_u16_le(self.value);
        (self.value / 2^n) % 2
    }
    fn getBits(&self, start: u16, length: u16) -> u16{
        let hw = HalfWord::from_u16_le(self.value);
        (self.value / 2^start) % 2^length
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
    //initializing video registers
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
    //setting up timing
    let mut currentCycle = elapsed.as_nanos() % CYCLE_TIME as u128;
    if(currentCycle > V_BLANK_TIME as u128){
        status.setBit(1, VBlank_BIT, mem);
        return;
    }
    else{
        status.setBit(0, VBlank_BIT, mem);
    }
    let mut currentScanline = elapsed.as_nanos() % SCANLINE_TIME as u128;
    if(currentCycle > H_BLANK_TIME as u128){
        status.setBit(1, HBlank_BIT, mem);
    }
    else{
        status.setBit(0, HBlank_BIT, mem);
    }

    let mut screen = RgbImage::new(240, 160);
    //display mode 1
    if(control.getBits(0, 2) == 1){
        let mut priorities:[usize; 4] = [0,0,0,0];
        for x in 0..4{
            let mut bgControl = Register {
                value: mem.get_halfword(BG_CNTRL_ADDR[x]).little_endian(),
                address: BG_CNTRL_ADDR[x]
            };
            let layer: u16 = bgControl.getBits(BG_PRIORITY_START_BIT as u16, 2);
            priorities[layer as usize] = x;
        }

        for i in 0..4{
            addBGLayer(priorities[(4 - i - 1) as usize], &mut screen, mem);
        }
    }

}

pub fn addBGLayer(bgNum: usize, screen: &mut RgbImage, mem: &mut Mem){
    
        let mut bgControl = Register {
            value: mem.get_halfword(BG_CNTRL_ADDR[bgNum]).little_endian(),
            address: BG_CNTRL_ADDR[bgNum]
        };
        let xOffset: usize = mem.get_halfword(BG_HORIZONTAL_OFFSET_ADDR[bgNum]).little_endian() as usize;
        let yOffset: usize = mem.get_halfword(BG_VERTICAL_OFFSET_ADDR[bgNum]).little_endian() as usize;
        let charBase: usize = bgControl.getBits(2, 2) as usize;
        let screenBase: usize = bgControl.getBits(8, 5) as usize;
        let sizeMode: usize = bgControl.getBits(14,2) as usize;
        let colorMode: usize = bgControl.getBit(PALETTES_BIT as u16) as usize;
        let startTile = (yOffset/8) * 32 + (xOffset/8);
        
        for x in 0..240 {
            for y in 0..160 {
                let mut currentTile: usize = 0;
                if(sizeMode == 0){
                    currentTile = (((yOffset + y) % 256)/8) * 32 + (((xOffset + x) % 256)/8);
                }
                else if(sizeMode == 1){
                    if(((xOffset + x) % 512) < 256){
                        currentTile = (((yOffset + y) % 256)/8) * 32 + (((xOffset + x) % 512)/8);
                    }
                    else{
                        currentTile = 1024 + (((yOffset + y) % 256)/8) * 32 + (((xOffset + x) % 256)/8);
                    }
                }
                else if(sizeMode == 2){
                    currentTile = (((yOffset + y) % 512)/8) * 32 + (((xOffset + x) % 256)/8);
                }
                else if(sizeMode == 3){
                    let currentTile = (((yOffset + y) % 512)/8) * 32 + (((xOffset + x) % 512)/8);
                }
                let xWithinTile: u8 = (x % 8) as u8;
                let yWithinTile: u8 = (y % 8) as u8;
                let tileMapAddr: usize = TILE_DATA_ADDR + screenBase * 2048;
                let currentPixelNum: usize = (yWithinTile * 8 + xWithinTile) as usize;
                //256 color mode
                if(colorMode == 1){
                    let currentTileAddr = tileMapAddr + 2 * currentTile;
                    let currentPixelData = mem.get_halfword(TILE_DATA_ADDR + charBase * 0x4000 + currentTileAddr * 0x40 + currentPixelNum).little_endian();
                    let blueComp: u16 = (currentPixelData / 2^10)  % (2^5);
                    let greenComp: u16 = (currentPixelData / 2^5)  % (2^5); 
                    let redComp: u16 = (currentPixelData)  % (2^5);
                    screen.put_pixel(x as u32, y as u32, Rgb([redComp as u8, greenComp as u8, blueComp as u8]));
                } //16 color mode
                else{
                        
                }
                    
                
            }
        }
}