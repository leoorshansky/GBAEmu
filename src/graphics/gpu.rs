#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_mut)]
#![allow(unused_parens)]
#![allow(unused_variables)]
use std::{time::Duration, u16, u8, usize};
use image::{Rgb, RgbImage};

use crate::arm::mem::Mem;
use crate::arm::common::{HalfWord};

const PRAM_START: usize = 0x05000000;
const PRAM_END: usize = 0x050003FF;
const VRAM_START: usize = 0x06000000;
const VRAM_END: usize = 0x06017FFF;
const OAM_START: usize = 0x07000000;
const OAM_END: usize = 0x070003FF;

//Control Register Information
const REG_DISPCNT_ADDR: usize = 0x04000000;
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
const REG_DISPSTAT_ADDR: usize = 0x04000004;
const VBlank_BIT: u8 = 0;
const HBlank_BIT: u8 = 1;
const VCountTrigger_BIT: u8 = 2;
const VBlankInterruptRequest_BIT: u8 = 3;
const HBlankInterruptRequest_BIT: u8 = 4;
const VCountInterruptRequest_BIT: u8 = 5;
const VCountTriggerValue_START_BIT: u8 = 8;
const VCountTriggerValue_END_BIT: u8 = 15;
//VCounter Register Information
const REG_VCOUNT_ADDR: usize = 0x04000006;
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
//Sprite Tile Data/PRAM information
const SPRITE_TILE_DATA_ADDR:usize = 0x06010000;
const SPRITE_PRAM_ADDR:usize = 0x05000200;
//Bitmap addresses
const BITMAP_DATA_ADDR : usize = 0x06000000;
const MODE3_DATA_ENDADDR : usize = 0x06013FFF;
const MODE45_DATA_ENDADDR : usize = 0x06009FFF;
//OBJ Tile Addresses 
const OBJ_DATA_ADDR : usize = 0x06014000;
const OBJ_DATA_ENDADDR : usize = 0x06017FFF;
//BG Rotation/Scaling Register Addresses
const BG_X_SCALE_ADDR: [usize; 2] = [0x4000020, 0x4000030];
const BG_Y_SCALE_ADDR: [usize; 2] = [0x4000026, 0x4000036];
const BG_X_SHEAR_ADDR: [usize; 2] = [0x4000022, 0x4000032];
const BG_Y_SHEAR_ADDR: [usize; 2] = [0x4000024, 0x4000034];
const FRACTIONAL_PART_START_BIT: usize = 0;
const FRACTIONAL_PART_END_BIT: usize = 7;
const INTEGER_PART_START_BIT: usize = 8;
const INTEGER_PART_END_BIT: usize = 14;
const SIGN_BIT: usize = 15;
const BG_AFFINE_HORIZONTAL_OFFSET: [usize; 2] = [0x4000028, 0x4000038];
const BG_AFFINE_VERTICAL_OFFSET: [usize; 2] = [0x400002C, 0x400003C];
const FRACTIONAL_PART_OFFSET_START_BIT: usize = 0;
const FRACTIONAL_PART_OFFSET_END_BIT: usize = 7;
const INTEGER_PART_OFFSET_START_BIT: usize = 8;
const INTEGER_PART_OFFSET_END_BIT: usize = 26;
const SIGN_OFFSET_BIT: usize = 27;


pub struct Register{
    value: u16,
    address: usize,
}



impl Register{
    fn getValue(&self) -> u16{
       self.value
    }
    fn getBit(&self, n: u16) -> u16{
        (self.value / 2^n) % 2
    }
    fn getBits(&self, start: u16, length: u16) -> u16{
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
    let mut prioritySprites: Vec<Vec<usize>> = Vec::new();
    for i in 0..4{
        prioritySprites.push(Vec::new());
    }
    for x in 0..128{
        let attr0 = mem.get_word(OAM_START + 8 * x +  0 * 2).little_endian();
        let attr2 = mem.get_word(OAM_START + 8 * x +  2 * 2).little_endian();
        if((attr0 / 2^7) % 2^2 != 2){
            let priority = (attr2 / (2^9)) % 2^2;
            prioritySprites[priority as usize].push(x);
        }
    }
    //display mode 1
    if(control.getBits(VideoMode_START_BIT as u16, 2) == 1){
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
            if(control.getBit((BG0Display_BIT + i) as u16) == 1){
                addBGTileLayer(priorities[(4 - i - 1) as usize], &mut screen, mem);
                for sprite in prioritySprites[i as usize].iter() {
                    let x = *sprite;
                    let controlCopy = Register {
                        value: mem.get_halfword(REG_DISPCNT_ADDR).little_endian(),
                        address: REG_DISPCNT_ADDR,
                    };
                    drawTiledSprite(x, &mut screen, mem, controlCopy);
                }
        }
    }
    if(control.getBits(VideoMode_START_BIT as u16, 2) == 3){
        if(control.getBit(BG2Display_BIT as u16) == 1){
            addBGBitmapLayer(&mut screen, mem);
        }
    }

    }
}

pub fn addBGTileLayer(bgNum: usize, screen: &mut RgbImage, mem: &mut Mem) -> (){
    
    for x in 0..240 {
        for y in 0..160 {
            let currentPixelColor = getCurrentPixelColor(x, y, bgNum, false, mem);
            let blueComp: u16 = (currentPixelColor / 2^10)  % (2^5);
            let greenComp: u16 = (currentPixelColor / 2^5)  % (2^5); 
            let redComp: u16 = (currentPixelColor)  % (2^5);
            screen.put_pixel(x as u32, y as u32, Rgb([redComp as u8, greenComp as u8, blueComp as u8]));                
        }
    }
    
}

pub fn drawTiledSprite(spriteNum: usize, screen: &mut RgbImage, mem: &mut Mem, control: Register) -> (){
    
    let attr0 = mem.get_word(OAM_START + 8 * spriteNum +  0 * 2).little_endian();
    let attr2 = mem.get_word(OAM_START + 8 * spriteNum + 4 * 2).little_endian();
    let attr1 = mem.get_word(OAM_START + 8 * spriteNum + 2 * 2).little_endian();
    let attr3 = mem.get_word(OAM_START + 8 * spriteNum + 6 * 2).little_endian();
    let sizeMode = (attr0 / (2^14));
    let baseTile = attr2 % (2^9);
    let colorMode = (attr0 / (2^13)) % 2;
    let tileIndexingMode = control.getBit(ObjectMappingMode_BIT as u16);
    let xCoord = attr1 % (2^8);
    let yCoord = attr0 % (2^8);
    let horizontalFlip = (attr1 / (2^12)) % 2;
    let verticalFlip = (attr2 / (2^13)) % 2;

    let spriteShape = (attr0 / (2^14));
    let spriteSize = (attr1 / (2^14));

    let mut xDim = 0;
    let mut yDim = 0;
    
    if(spriteShape ==  0){
        if(spriteSize == 0){
            xDim = 8;
            yDim = 8;
        }
        else if(spriteSize == 1){
            xDim = 16;
            yDim = 16;
        }
        else if(spriteSize == 2){
            xDim = 32;
            yDim = 32;
        }
        else if(spriteSize == 3){
            xDim = 64;
            yDim = 64;
        }
    }
    else if(spriteShape == 1){
        if(spriteSize == 0){
            xDim = 16;
            yDim = 8;
        }
        else if(spriteSize == 1){
            xDim = 32;
            yDim = 8;
        }
        else if(spriteSize == 2){
            xDim = 32;
            yDim = 16;
        }
        else if(spriteSize == 3){
            xDim = 64;
            yDim = 32;
        }
    }
    else if(spriteShape == 2){
        if(spriteSize == 0){
            xDim = 8;
            yDim = 16;
        }
        else if(spriteSize == 1){
            xDim = 8;
            yDim = 32;
        }
        else if(spriteSize == 2){
            xDim = 16;
            yDim = 32;
        }
        else if(spriteSize == 3){
            xDim = 32;
            yDim = 64;
        }
    }
    let mut lastX = xCoord + xDim;
    let mut lastY = yCoord + yDim;
    if(xCoord + xDim > 240){
        lastX = 240;
    }
    if(yCoord + yDim > 160){
        lastY = 160;
    }
    for x in xCoord..lastX{
        for y in yCoord..lastY{
            let currentTile = (y / 8) * (xDim / 8) + (x/8);
            let mut currentTileWithinMemory = 0;
            if(tileIndexingMode == 1){
                if(colorMode == 1){
                    currentTileWithinMemory = baseTile + (currentTile / (xDim / 8)) * 32 + (currentTile % (xDim/8)) * 2;
                }
                else{
                    currentTileWithinMemory = baseTile + (currentTile / (xDim / 8)) * 32 + (currentTile % (xDim/8));
                }
            }
            else if(tileIndexingMode == 0){
                currentTileWithinMemory = baseTile + currentTile;
            }
            
            let xWithinTile = x % 8;
            let yWithinTile = y % 8;
            let mut currentPixelData = 0;
            let mut currentPixelColor = 0;
            let currentPixelNum = yWithinTile * 8 + xWithinTile;
            //256-color palette
            if(colorMode == 1){
                currentPixelData = mem.get_byte(SPRITE_TILE_DATA_ADDR + (0x20 * currentTileWithinMemory + currentPixelNum) as usize); 
                currentPixelColor = mem.get_halfword(SPRITE_PRAM_ADDR + (2 * currentPixelData) as usize).little_endian();
            }
            //16 color palette
            else if(colorMode == 0){
                let paletteNum = attr2 / (2^12);
                currentPixelData = mem.get_byte(SPRITE_TILE_DATA_ADDR + (0x20 * currentTileWithinMemory + currentPixelNum / 2) as usize);
                if(currentPixelNum % 2 == 0){
                    currentPixelData = mem.get_byte(SPRITE_TILE_DATA_ADDR + (0x20 * currentTileWithinMemory + currentPixelNum) as usize)/ 2^4;
                }
                else{
                    currentPixelData = mem.get_byte(SPRITE_TILE_DATA_ADDR + (0x20 * currentTileWithinMemory + currentPixelNum) as usize) % 2^4;
                }
                currentPixelColor = mem.get_halfword(SPRITE_PRAM_ADDR + (paletteNum * 32 + (currentPixelData * 2) as u32) as usize).little_endian();

            }
            let blueComp: u16 = (currentPixelColor / 2^10)  % (2^5);
            let greenComp: u16 = (currentPixelColor / 2^5)  % (2^5); 
            let redComp: u16 = (currentPixelColor)  % (2^5);
            let mut screenX = x;
            let mut screenY = y;
            if(horizontalFlip == 1){
                screenX = lastX - (x - xCoord + 1);
            }
            if(verticalFlip == 1){
                screenY = lastY - (y - yCoord + 1);
            }
            screen.put_pixel(screenX as u32, screenY as u32, Rgb([redComp as u8, greenComp as u8, blueComp as u8]));  
        }
    }
}



pub fn addBGBitmapLayer(screen: &mut RgbImage, mem: &mut Mem) -> (){
    let mut bgControl = Register {
        value: mem.get_halfword(BG_CNTRL_ADDR[2]).little_endian(),
        address: BG_CNTRL_ADDR[2]
    };
    let xOffset: usize = mem.get_halfword(BG_HORIZONTAL_OFFSET_ADDR[2]).little_endian() as usize;
    let yOffset: usize = mem.get_halfword(BG_VERTICAL_OFFSET_ADDR[2]).little_endian() as usize;
    for x in 0..240{
        for y in 0..160{
            let currentPixelColor = mem.get_halfword(TILE_DATA_ADDR + ((yOffset + y) * 160 + (xOffset + x)) * 2).little_endian(); 
            let blueComp: u16 = (currentPixelColor / 2^10)  % (2^5);
            let greenComp: u16 = (currentPixelColor / 2^5)  % (2^5); 
            let redComp: u16 = (currentPixelColor)  % (2^5);
            screen.put_pixel(x as u32, y as u32, Rgb([redComp as u8, greenComp as u8, blueComp as u8]));      

        }
    }

}

pub fn getCurrentPixelColor(x: usize, y: usize, bgNum: usize, affine: bool, mem: &mut Mem) -> (u16){
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

    let mut currentTile: usize = 0;
    let mut xWithinTile = 0;
    let mut yWithinTile = 0;
    if(affine){

    }
    else{
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
        xWithinTile= (x % 8) as u8;
        yWithinTile= (y % 8) as u8;
    }
    let tileMapAddr: usize = TILE_DATA_ADDR + screenBase * 2048;   
    let currentTileAddr = tileMapAddr + 2 * currentTile;
    let currentTileData = mem.get_halfword(currentTileAddr).little_endian();
    let horizontalFlipping = (currentTileData / 2^10) % 2;
    let verticalFlipping = (currentTileData / 2^11) % 2;
    if(affine){
        
    }
    else{
        if(horizontalFlipping == 1){
            xWithinTile = 7 - xWithinTile;
        }
        if(verticalFlipping == 1){
            yWithinTile = 7 - yWithinTile;
        }
    }
    let currentPixelNum: usize = (yWithinTile * 8 + xWithinTile) as usize;
    let mut currentPixelData = 0;
    let mut currentPixelColor: u16 = 0;
    //256 color palette
    if(colorMode == 1){
        currentPixelData = mem.get_byte(TILE_DATA_ADDR + charBase * 0x4000 + (currentTileData % (2^8)) as usize * 0x40 + currentPixelNum);
        currentPixelColor= mem.get_halfword(PRAM_START + (currentPixelData * 2) as usize).little_endian();
    }
    //16 color palette
    else{
        currentPixelData = mem.get_byte(TILE_DATA_ADDR + charBase * 0x4000 + (currentTileData % (2^8)) as usize * 0x20 + currentPixelNum / 2);
        if(currentPixelNum % 2 ==0){
            currentPixelData = currentPixelData / 2^4;
        }
        else{
            currentPixelData = currentPixelData % (2^4);
        }
        currentPixelColor= mem.get_halfword(PRAM_START + ((currentTileData / 2^12) * 32) as usize + (currentPixelData * 2) as usize).little_endian();
    }
    return currentPixelColor;
    
}
