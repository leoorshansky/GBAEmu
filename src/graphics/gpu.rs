#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_mut)]
#![allow(unused_parens)]
#![allow(unused_variables)]
use std::{u16, u8, usize};
use gdk_pixbuf::{Pixbuf, Colorspace};
use image::{RgbImage, Rgb};
use std::time::{Duration, Instant};
use std::fmt;

use crate::arm::{cpu::{Cpu}, mem::{Mem, Interrupt}};
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
const FRAME_CYCLES: usize = 280_896;
const SCANLINE_CYCLES: usize = 1232;
const H_BLANK_CYCLES: usize = 1004;
const V_BLANK_CYCLES: usize = 197_120;
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


pub struct Register {
    value: u16,
    address: usize,
}



impl Register{
    fn getValue(&mut self, mem: &Mem) -> u16{
        self.value = mem.get_halfword(self.address).little_endian();
        self.value
    }
    fn getBit(&mut self, n: u16, mem: &Mem) -> u16{
        self.value = mem.get_halfword(self.address).little_endian();
        self.value >> n & 1
    }
    fn getBits(&mut self, start: u16, num_bits: u16, mem: &Mem) -> u16 {
        self.value = mem.get_halfword(self.address).little_endian();
        (self.value >> start) % (1 << num_bits)
    }
    fn setValue(&mut self, v: u16, mem: &mut Mem) {
        self.value = v;
        let hw = HalfWord::from_u16_le(self.value);
        mem.set_halfword(self.address, hw);        
    }
    fn setBit(&mut self, v: u8, n: u8, mem: &mut Mem) {
        if v == 0 {
            self.value &= !(1 << n);
        }
        else if v == 1 {
            self.value |= 1 << n;
        }
        else{
            //this should never occur
            println!("your mom");
        }
        let hw = HalfWord::from_u16_le(self.value);
        mem.set_halfword(self.address, hw);
    }
}


pub fn draw(mem: &mut Mem, cycle: usize) -> Option<Pixbuf> {
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
    let oldvCounter = mem.get_byte(0x4000006);
    let currentLine = (((cycle % FRAME_CYCLES) as f64 / FRAME_CYCLES as f64) * 228f64 ) as u16;
    vCounter.setValue(currentLine, mem);
    //if currentLine != oldvCounter as u16 { println!("VCOUNTER IS {}", mem.get_byte(0x4000006)) };
    if(vCounter.getValue(mem) == status.getBits(VCountTriggerValue_START_BIT as u16, 8, mem)){
        if(status.getBit(VCountInterruptRequest_BIT as u16, mem) == 1){
            mem.request_irq(Interrupt::VCounter);
        }
        status.setBit(1, VCountTrigger_BIT, mem);
    }
    let mut blanking = false;
    if(cycle % SCANLINE_CYCLES > H_BLANK_CYCLES){
        blanking = true;
        if(status.getBit(HBlankInterruptRequest_BIT as u16, mem) == 1 && status.getBit(HBlank_BIT as u16, mem) == 0){
            mem.request_irq(Interrupt::HBlank);
        }
        status.setBit(1, HBlank_BIT, mem);
    }
    else{
        status.setBit(0, HBlank_BIT, mem);
    }
    if(cycle % FRAME_CYCLES > V_BLANK_CYCLES){
        blanking = true;
        if(status.getBit(VBlankInterruptRequest_BIT as u16, mem) == 1 && status.getBit(VBlank_BIT as u16, mem) == 0){
            mem.request_irq(Interrupt::VBlank);
        }
        status.setBit(1, VBlank_BIT, mem);
    }
    else{
        status.setBit(0, VBlank_BIT, mem);
    }
    if blanking {
        return None;
    }
    
    let mut screen = Pixbuf::new(Colorspace::Rgb, false, 8, 960, 640).unwrap();
    screen.fill(0xFFFFFF00);
    //let mut screen: [u8; 1843200] = [0; 1843200];
    //let mut screen: [u8; 115200] = [0; 115200];


    //display mode 1
    if(true){
        let mut prioritySprites: Vec<Vec<usize>> = Vec::new();
        for i in 0..4{
            prioritySprites.push(Vec::new());
        }
        for x in 0..128{
            let attr0 = mem.get_halfword(OAM_START + 8 * x +  0 * 2).little_endian();
            let attr2 = mem.get_halfword(OAM_START + 8 * x +  2 * 2).little_endian();
            
            if((attr0 >> 8) & 0b11 != 2){
                let priority = (attr2 >> 10) & 0b11;
                prioritySprites[priority as usize].push(x);
            }
        }
        let mut priorities:[usize; 4] = [5,5,5,5];
        for x in 0..4{
            let mut bgControl = Register {
                value: mem.get_halfword(BG_CNTRL_ADDR[x]).little_endian(),
                address: BG_CNTRL_ADDR[x]
            };
            let layer: u16 = bgControl.getBits(BG_PRIORITY_START_BIT as u16, 2, &mem);
            if(control.getBit((BG0Display_BIT + x as u8) as u16, &mem) == 1){
                priorities[layer as usize] = x;
            }
        }

        for i in 0..4{
            if(priorities[4-i-1] != 5){
                if(control.getBits(VideoMode_START_BIT as u16, 2, mem) == 0 || control.getBits(VideoMode_START_BIT as u16, 2, mem) == 1){
                    addBGTileLayer(priorities[(4 - i - 1) as usize], &mut screen, mem);
                }
            }
            for sprite in prioritySprites[(4 - i - 1)as usize].iter().rev() {
                let x = *sprite;
                let controlCopy = Register {
                    value: mem.get_halfword(REG_DISPCNT_ADDR).little_endian(),
                    address: REG_DISPCNT_ADDR,
                };
                if(x != 9){
                    drawTiledSprite(x, &mut screen, mem, controlCopy);
                }
            }
        }
    }
    
    // if(control.getBits(VideoMode_START_BIT as u16, 2) == 3){
    //     if(control.getBit(BG2Display_BIT as u16) == 1){
    //         addBGBitmapLayer(&mut screen, mem);
    //     }
    // }
    //let v = Pixbuf::from_mut_slice(&mut screen, Colorspace::Rgb, false, 8, 240, 160, 720);
    return Some(screen);
}


pub fn addBGTileLayer(bgNum: usize, screen: &mut Pixbuf, mem: &mut Mem) {
    for y in 0..160 {
        for x in 0..240 {
            let currentPixelColor = getCurrentPixelColor(x, y, bgNum, false, mem);
            let blueComp: u16 = (currentPixelColor >> 10) & 0b11111;
            let greenComp: u16 = (currentPixelColor >> 5) & 0b11111; 
            let redComp: u16 = currentPixelColor & 0b11111;
            // screen[(720*y + 3*x) as usize] = (redComp as u8) << 3;
            // screen[((720*y + 3*x) + 1) as usize] = (greenComp as u8) << 3;
            // screen[((720*y + 3*x) + 2) as usize] = (blueComp as u8) << 3;
            
            for x1 in 0..4{
                for y1 in 0..4{
                    // screen[((960*(4*y+y1)  + (4*x+x1)) * 3) as usize] = (redComp as u8) << 3;
                    // screen[((960*(4*y+y1)  + (4*x+x1)) * 3 + 1) as usize] = (greenComp as u8) << 3;
                    // screen[((960*(4*y+y1)  + (4*x+x1)) * 3 + 2) as usize] = (blueComp as u8) << 3;
                    if(redComp != 0 || greenComp != 0 || blueComp != 0){
                        screen.put_pixel((4*x + x1) as u32, (4 * y + y1) as u32, (redComp as u8) << 3 , (greenComp as u8) << 3, (blueComp as u8) << 3, 1);
                    }
                }
            } 
                      
        }
    }
    
}

pub fn drawTiledSprite(spriteNum: usize, screen: &mut Pixbuf, mem: &mut Mem, mut control: Register) {
    
    let attr0 = mem.get_halfword(OAM_START + 8 * spriteNum + 0 * 2).little_endian();
    let attr1 = mem.get_halfword(OAM_START + 8 * spriteNum + 1 * 2).little_endian();
    let attr2 = mem.get_halfword(OAM_START + 8 * spriteNum + 2 * 2).little_endian();
    let attr3 = mem.get_halfword(OAM_START + 8 * spriteNum + 3 * 2).little_endian();
    let sizeMode = attr0 >> 14;
    let doubleSize = attr0 >> 8 & 0b11 == 3;
    let baseTile = attr2 & 0b11111_11111;
    let colorMode = (attr0 >> 13) & 0b1;
    let tileIndexingMode = if control.getBit(ObjectMappingMode_BIT as u16, mem) == 1 { 0 } else { 1 };
    let mut xCoord = attr1 & 0b11111_1111;
    let yCoord = attr0 & 0b1111_1111;
    let horizontalFlip = (attr1 >> 12) & 0b1;
    let verticalFlip = (attr2 >> 13) & 0b1;

    let spriteShape = attr0 >> 14;
    let spriteSize = attr1 >> 14;

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

    if doubleSize {
        lastX += xDim;
        lastY += yDim;
    }
    
    // if(xCoord + xDim > 240){
    //     lastX = 240;
    // }
    // if(yCoord + yDim > 160){
    //     lastY = 160;
    // }
    // if xCoord > 240 || yCoord > 160 {
    //     return;
    // }
    for mut x in xCoord..lastX{
        for mut y in yCoord..lastY{

            let mut spriteX = x - xCoord;
            let mut spriteY = y - yCoord;

            if((attr0 >> 8) & 0b11 == 1 || (attr0 >> 8) & 0b11 == 3){

                let centeredX = spriteX as i16 - xDim as i16;
                let centeredY = spriteY as i16 - yDim as i16;

                let affineIndex = (attr1 >> 9) & 0b11111;
                let pA = mem.get_halfword(OAM_START + (0x20 * affineIndex) as usize + 0x6).little_endian();
                let pA = if pA >> 15 == 1 { -1. } else { 1. } * (pA as f32 / 256f32);
                let pB = mem.get_halfword(OAM_START + (0x20 * affineIndex) as usize + 0xE).little_endian();
                let pB = if pB >> 15 == 1 { -1. } else { 1. } * (pB as f32 / 256f32);
                let pC = mem.get_halfword(OAM_START + (0x20 * affineIndex) as usize + 0x16).little_endian();
                let pC = if pC >> 15 == 1 { -1. } else { 1. } * (pC as f32 / 256f32);
                let pD = mem.get_halfword(OAM_START + (0x20 * affineIndex) as usize + 0x1E).little_endian();
                let pD = if pD >> 15 == 1 { -1. } else { 1. } * (pD as f32 / 256f32);
                
                spriteX = ((xDim / 2) as i16 + (pA * centeredX as f32) as i16 + (pB * centeredY as f32) as i16) as u16;
                spriteY = ((yDim / 2) as i16 + (pC * centeredX as f32) as i16 + (pD * centeredY as f32) as i16) as u16;
            }

            if (spriteX as i16) < 0 || (spriteY as i16) < 0 || spriteX > xDim || spriteY > yDim {
                continue;
            }
            
            let currentTile = (spriteY / 8) * (xDim / 8) + (spriteX/8);
            let mut currentTileWithinMemory = 0;
            if(tileIndexingMode == 1){
                if(colorMode == 1){
                    currentTileWithinMemory = baseTile + (spriteY / 8) * 32 + (spriteX / 8) * 2;
                }
                else{
                    currentTileWithinMemory = baseTile + (spriteY/8) * 32 + (spriteX / 8);
                }
            }
            else if(tileIndexingMode == 0){
                if colorMode == 1 {
                    currentTileWithinMemory = baseTile + currentTile * 2;
                }
                else{
                    currentTileWithinMemory = baseTile + currentTile;
                }
    
            }
            
            let xWithinTile = spriteX % 8;
            let yWithinTile = spriteY % 8;
            let mut currentPixelData = 0;
            let mut currentPixelColor = 0;
            let currentPixelNum = yWithinTile * 8 + xWithinTile;
            //256-color palette
            if(colorMode == 1){
                currentPixelData = mem.get_byte(SPRITE_TILE_DATA_ADDR + (0x20 * currentTileWithinMemory as usize + currentPixelNum as usize) as usize); 
                currentPixelColor = mem.get_halfword(SPRITE_PRAM_ADDR + (2 * currentPixelData as usize) as usize).little_endian();
            }
            //16 color palette
            else if(colorMode == 0){
                let paletteNum = attr2 >> 12;
                if(currentPixelNum % 2 == 0){
                    currentPixelData = mem.get_byte(SPRITE_TILE_DATA_ADDR + (0x20 * currentTileWithinMemory as usize + (currentPixelNum/2) as usize) as usize) & 0b1111;
                } else{
                    currentPixelData = mem.get_byte(SPRITE_TILE_DATA_ADDR + (0x20 * currentTileWithinMemory as usize + (currentPixelNum/2) as usize) as usize) >> 4;
                }
                currentPixelColor = mem.get_halfword(SPRITE_PRAM_ADDR + (paletteNum * 32 + (currentPixelData * 2) as u16) as usize).little_endian();

            }
            let blueComp: u16 = currentPixelColor >> 10 & 0b11111;
            let greenComp: u16 = currentPixelColor >> 5 & 0b11111; 
            let redComp: u16 = (currentPixelColor) & 0b11111;
            
            let mut screenX = x;
            let mut screenY = y;
            if(horizontalFlip == 1){
                screenX = lastX - (x - xCoord + 1);
            }
            if(verticalFlip == 1){
                screenY = lastY - (y - yCoord + 1);
            }
            // screen[((240*(y)  + (x)) * 3) as usize] = (redComp as u8) << 3;
            // screen[((240*(y)  + (x)) * 3 + 1) as usize] = (greenComp as u8) << 3;
            // screen[((240*(y)  + (x)) * 3 + 2) as usize] = (blueComp as u8) << 3;
            if(currentPixelData == 0){
                continue;
            }
            if redComp != 0 || greenComp != 0 || blueComp != 0 {
                for x1 in 0..4{
                    for y1 in 0..4{
                        // screen[((960*(4*y+y1)  + (4*x+x1)) * 3) as usize] = (redComp as u8) << 3;
                        // screen[((960*(4*y+y1)  + (4*x+x1)) * 3 + 1) as usize] = (greenComp as u8) << 3;
                        // screen[((960*(4*y+y1)  + (4*x+x1)) * 3 + 2) as usize] = (blueComp as u8) << 3;
                        y = y % 256;
                        if(x < 240 && y < 160){
                            screen.put_pixel((4*x + x1) as u32, (4 * y + y1) as u32, (redComp as u8) << 3 , (greenComp as u8) << 3, (blueComp as u8) << 3, 1);                            
                        }                   
                    }
                }
            }
                         
        }
    }
}



pub fn addBGBitmapLayer(screen: &mut Pixbuf, mem: &mut Mem) {
    let mut bgControl = Register {
        value: mem.get_halfword(BG_CNTRL_ADDR[2]).little_endian(),
        address: BG_CNTRL_ADDR[2]
    };
    let xOffset: usize = mem.get_halfword(BG_HORIZONTAL_OFFSET_ADDR[2]).little_endian() as usize;
    let yOffset: usize = mem.get_halfword(BG_VERTICAL_OFFSET_ADDR[2]).little_endian() as usize;
    for x in 0..240{
        for y in 0..160{
            let currentPixelColor = mem.get_halfword(TILE_DATA_ADDR + ((yOffset + y) * 160 + (xOffset + x)) * 2).little_endian(); 
            let blueComp: u16 = (currentPixelColor >> 10) & 0b11111;
            let greenComp: u16 = (currentPixelColor >> 5) & 0b11111; 
            let redComp: u16 = currentPixelColor & 0b11111;
            // screen[((240*(y)  + (x)) * 3) as usize] = (redComp as u8) << 3;
            // screen[((240*(y)  + (x)) * 3 + 1) as usize] = (greenComp as u8) << 3;
            // screen[((240*(y)  + (x)) * 3 + 2) as usize] = (blueComp as u8) << 3;
            
            for x1 in 0..4{
                for y1 in 0..4{
                    // screen[((960*(4*y+y1)  + (4*x+x1)) * 3) as usize] = (redComp as u8) << 3;
                    // screen[((960*(4*y+y1)  + (4*x+x1)) * 3 + 1) as usize] = (greenComp as u8) << 3;
                    // screen[((960*(4*y+y1)  + (4*x+x1)) * 3 + 2) as usize] = (blueComp as u8) << 3;
                    if(redComp != 0 || greenComp != 0 || blueComp != 0){
                        screen.put_pixel((4*x + x1) as u32, (4 * y + y1) as u32, (redComp as u8) << 3 , (greenComp as u8) << 3, (blueComp as u8) << 3, 1);
                    }                }
            }
               
                         

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
    let charBase: usize = bgControl.getBits(2, 2, mem) as usize;
    let screenBase: usize = bgControl.getBits(8, 5, mem) as usize;
    let sizeMode: usize = bgControl.getBits(14,2, mem) as usize;
    let colorMode: usize = bgControl.getBit(PALETTES_BIT as u16, mem) as usize;
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
        xWithinTile= ((xOffset + x) % 8) as u8;
        yWithinTile= ((yOffset + y) % 8) as u8;
    }
    let tileMapAddr: usize = TILE_DATA_ADDR + screenBase * 2048;   
    let currentTileAddr = tileMapAddr + 2 * currentTile;
    let currentTileData = mem.get_halfword(currentTileAddr).little_endian();
    let horizontalFlipping = (currentTileData >> 10) % 2;
    let verticalFlipping = (currentTileData >> 11) % 2;
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
        currentPixelData = mem.get_byte(TILE_DATA_ADDR + charBase * 0x4000 + (currentTileData & 0xff) as usize * 0x40 + currentPixelNum);
        currentPixelColor= mem.get_halfword(PRAM_START + (currentPixelData * 2) as usize).little_endian();
    }
    //16 color palette
    else{
        currentPixelData = mem.get_byte(TILE_DATA_ADDR + charBase * 0x4000 + (currentTileData & 0xff) as usize * 0x20 + currentPixelNum / 2);
        if(currentPixelNum % 2 ==0){
            currentPixelData &= 0b1111;
        }
        else{
            currentPixelData >>= 4;
        }
        currentPixelColor= mem.get_halfword(PRAM_START + ((currentTileData >> 12) * 32) as usize + (currentPixelData * 2) as usize).little_endian();
    }

    currentPixelColor
}
