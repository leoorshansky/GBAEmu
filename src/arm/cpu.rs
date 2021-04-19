use std::{io::Read, thread::current};
use anyhow::Result;

use crate::mem::Mem;

static REG_CPSR: usize = 31;
static REG_SPSR_FIQ: usize = 32;
static REG_SPSR_SVC: usize = 33;
static REG_SPSR_ABT: usize = 34;
static REG_SPSR_IRQ: usize = 35;
static REG_SPSR_UND: usize = 36;
static BIT_V: u8 = 28;
static BIT_C: u8 = 29;
static BIT_Z: u8 = 30;
static BIT_N: u8 = 31;
static NOP: usize = 0b1110_0010_1000_0000_0000_0000_0000_0000;

pub struct Cpu {
    ram: Mem,
    regs: [u32; 37],
    decode_stage: u32,
    execute_stage: u32
}

#[derive(Copy, Clone)]
enum Mode {
    User,
    Svc,
    Irq,
    Fiq,
    Und,
    Abt,
    Sys
}

#[derive(Copy, Clone)]
enum State {
    Arm,
    Thumb
}
 
impl Cpu {
    pub fn step(&mut self) {
        let state = self.get_state();
        let mode = self.get_mode();

        let instruction = self.execute_stage;
        self.execute_stage = self.decode_stage;
        self.decode_stage = self.ram.get_word(self.regs[15] as usize).little_endian();

        if let State::Arm = state {
            let sets_flags = 1 == instruction >> 20 & 1;
            let immediate = 1 == instruction >> 25 & 1;
            let opcode = instruction >> 21 & 0b111_1111; // Bits 21 through 27

            let rn = self.get_register(mode, instruction >> 16 & 0b1111);
            let rd_index = self.get_register_index(mode, instruction >> 12 & 0b1111);
            let rs = self.get_register(mode, instruction >> 8 & 0b1111);
            let rm = self.regs[(instruction & 0b1111) as usize];

            let mut shifter_carry = self.get_status_bit(BIT_C);

            let operand2 = if immediate {
                let rotate_amount = (instruction >> 8 & 0b1111) * 2;
                (instruction & 0xff) >> rotate_amount | (instruction & 0xff) << (32 - rotate_amount)
            } else {
                let rm = if instruction & 0b1111 == 0b1111 { rm + 4 } else { rm }; // PC is +4 if there is a register-specified shift
                let shift_amount = if 1 == (instruction >> 4) & 1 { rs & 0xff } else { instruction >> 7 & 0b11111 };
                if 1 == (instruction >> 4) & 1 && shift_amount == 0 {
                    rm
                } else {
                    match instruction >> 5 & 0b11 {
                        0b00 => { // Logical Shift Left
                            if shift_amount == 0 {
                                rm
                            } else if shift_amount == 32 {
                                shifter_carry = 1 == rm & 1;
                                0
                            } else if shift_amount > 32 {
                                shifter_carry = false;
                                0
                            } else {
                                shifter_carry = 1 == rm >> (32 - shift_amount) & 1;
                                rm << shift_amount
                            }
                        }
                        0b01 => { // Logical Shift Right
                            if shift_amount == 0 || shift_amount == 32 {
                                shifter_carry = 1 == rm >> 31;
                                0
                            } else if shift_amount > 32{
                                shifter_carry = false;
                                0
                            } else {
                                shifter_carry = 1 == rm >> (shift_amount - 1) & 1;
                                rm >> shift_amount
                            }
                        }
                        0b10 => { // Arithmetic Shift Right
                            if shift_amount == 0 || shift_amount >= 32 {
                                shifter_carry = 1 == rm >> 31;
                                if 1 == rm >> 31 { !0 } else { 0 }
                            } else {
                                shifter_carry = 1 == rm >> (shift_amount - 1) & 1;
                                if 1 == rm >> 31 {
                                    rm >> shift_amount | !(!0 >> shift_amount)
                                } else {
                                    rm >> shift_amount
                                }
                            }
                        }
                        0b11 => { // Rotate Right
                            if shift_amount == 0 {
                                shifter_carry = 1 == rm & 1;
                                if self.get_status_bit(BIT_C) { rm >> 1 | 1 << 31} else { rm >> 1}
                            } else if shift_amount == 32 {
                                shifter_carry = 1 == rm >> 31;
                                rm
                            } else {
                                shifter_carry = 1 == rm >> (shift_amount % 32 - 1) & 1;
                                rm >> (shift_amount % 32) | rm << (32 - shift_amount % 32)
                            }
                        }
                        _ => panic!()
                    }
                }
            };

            let mut signed_result = 0; // Decides sticky overflow flag
            let mut c = shifter_carry; // Carry flag

            if self.should_execute_arm(instruction) {
                if opcode >> 5 == 0 { // Data Processing
                    let mut write_back = true;
                    let result = match opcode & 0b1111 {
                        0b0000 => { // AND
                            rn & operand2
                        }
                        0b0001 => { // EOR
                            rn ^ operand2
                        }
                        0b0010 => { // SUB
                            signed_result = rn as i64 - operand2 as i64;
                            c = signed_result >= 0;
                            rn - operand2
                        }
                        0b0011 => { // RSB
                            signed_result = operand2 as i64 - rn as i64;
                            c = signed_result >= 0;
                            operand2 - rn
                        }
                        0b0100 => { // ADD
                            signed_result = operand2 as i64 + rn as i64;
                            c = signed_result >= 1 << 32;
                            operand2 + rn
                        }
                        0b0101 => { // ADC
                            let carry_in = self.get_status_bit(BIT_C);
                            signed_result = operand2 as i64 + rn as i64 + carry_in as i64;
                            c = signed_result >= 1 << 32;
                            operand2 + rn + carry_in as u32
                        }
                        0b0110 => { // SBC
                            let carry_in = self.get_status_bit(BIT_C);
                            signed_result = rn as i64 - operand2 as i64 + carry_in as i64 - 1;
                            c = signed_result >= 0;
                            rn - operand2 + carry_in as u32 - 1
                        }
                        0b0111 => { // RSC
                            let carry_in = self.get_status_bit(BIT_C);
                            signed_result = operand2 as i64 - rn as i64 + carry_in as i64 - 1;
                            c = signed_result >= 0;
                            operand2 - rn + carry_in as u32 - 1
                        }
                        0b1000 => { // TST
                            write_back = false;
                            rn & operand2
                        }
                        0b1001 => { // TEQ
                            write_back = false;
                            rn ^ operand2
                        }
                        0b1010 => { // CMP
                            write_back = false;
                            signed_result = rn as i64 - operand2 as i64;
                            c = signed_result >= 0;
                            rn - operand2
                        }
                        0b1011 => { // CMN
                            write_back = false;
                            signed_result = operand2 as i64 + rn as i64;
                            c = signed_result >= 1 << 32;
                            operand2 + rn
                        }
                        0b1100 => { // ORR
                            rn | operand2
                        }
                        0b1101 => { // MOV
                            operand2
                        }
                        0b1110 => { // BIC
                            rn & !operand2
                        }
                        0b1111 => { // MVN
                            !operand2
                        }
                        _ => panic!()
                    };
                    if sets_flags && rd_index != 15 {
                        self.set_status_bit(BIT_C, c);
                        self.set_status_bit(BIT_N, 1 == result >> 31);
                        self.set_status_bit(BIT_Z, result == 0);
                        if signed_result >= 1 << 31 || signed_result < -(1 << 31) {
                            self.set_status_bit(BIT_V, true);
                        }
                    } else if sets_flags { // Transfer the SPSR
                        let current_mode_register = self.get_mode_status_register(mode);
                        self.regs[REG_CPSR] = current_mode_register;
                    }
                    if write_back {
                        self.regs[rd_index] = result;
                    }
                }
                let x = 6;
            }
        }
    }

    #[inline(always)]
    fn get_register(&self, mode: Mode, index: u32) -> u32 {
        self.regs[self.get_register_index(mode, index)]
    }

    #[inline(always)]
    fn get_register_index(&self, mode: Mode, index: u32) -> usize {
        let fiq = matches!(mode, Mode::Fiq);
        match index {
            8..=12 if fiq => index as usize - 8 + 16,
            13 | 14 => match mode {
                Mode::User | Mode::Sys => index as usize,
                Mode::Fiq => index as usize - 13 + 21,
                Mode::Svc => index as usize - 13 + 23,
                Mode::Abt => index as usize - 13 + 25,
                Mode::Irq => index as usize - 13 + 27,
                Mode::Und => index as usize - 13 + 29,
            }
            _ => index as usize
        }
    }

    #[inline(always)]
    fn set_status_bit(&mut self, bit: u8, high: bool) {
        if high {
            self.regs[REG_CPSR] |= 1 << bit;
        } else {
            self.regs[REG_CPSR] &= !(1 << bit);
        }
    }

    #[inline(always)]
    fn get_status_bit(&self, bit: u8) -> bool {
        1 == self.regs[REG_CPSR] >> bit & 1
    }

    #[inline(always)]
    fn get_mode_status_register(&self, mode: Mode) -> u32 {
        match mode {
            Mode::User | Mode::Sys => { self.regs[REG_CPSR] }
            Mode::Svc => { self.regs[REG_SPSR_SVC] }
            Mode::Irq => { self.regs[REG_SPSR_IRQ] }
            Mode::Fiq => { self.regs[REG_SPSR_FIQ] }
            Mode::Und => { self.regs[REG_SPSR_UND] }
            Mode::Abt => { self.regs[REG_SPSR_ABT] }
        }
    }

    #[inline(always)]
    fn should_execute_arm(&self, instruction: u32) -> bool {
        let v = 1 == self.regs[REG_CPSR] >> 28 & 1;
        let c = 1 == self.regs[REG_CPSR] >> 29 & 1;
        let z = 1 == self.regs[REG_CPSR] >> 30 & 1;
        let n = 1 == self.regs[REG_CPSR] >> 31;

        match instruction >> 28 {
            0b0000 => z, // EQ
            0b0001 => !z, // NE
            0b0010 => c, // CS/HS
            0b0011 => !c, // CC/LO
            0b0100 => n, // MI
            0b0101 => !n, // PL
            0b0110 => v, // VS
            0b0111 => !v, // VC
            0b1000 => c && !z, // HI
            0b1001 => !c || z, // LS
            0b1010 => n == v, // GE
            0b1011 => n != v, // LT
            0b1100 => !z && (n == v), // GT
            0b1101 => z || (n != v), // LE
            0b1110 => true,
            _ => panic!()
        }
    }

    #[inline(always)]
    fn get_mode(&self) -> Mode {
        match (self.regs[REG_CPSR] & 0b1_1111) as u8 {
            0b10000 => Mode::User,
            0b10001 => Mode::Fiq,
            0b10010 => Mode::Irq,
            0b10011 => Mode::Svc,
            0b10111 => Mode::Abt,
            0b11011 => Mode::Und,
            0b11111 => Mode::Sys,
            _ => panic!()
        }
    }

    #[inline(always)]
    fn get_state(&self) -> State {
        match (self.regs[REG_CPSR] & 0b100000) == 0 {
            true => State::Arm,
            false => State::Thumb
        }
    }
}