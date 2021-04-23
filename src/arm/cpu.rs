use crate::mem::Mem;

use super::common::{HalfWord, Word};

const REG_CPSR: usize = 31;
const REG_SPSR_FIQ: usize = 32;
const REG_SPSR_SVC: usize = 33;
const REG_SPSR_ABT: usize = 34;
const REG_SPSR_IRQ: usize = 35;
const REG_SPSR_UND: usize = 36;
const BIT_V: u8 = 28;
const BIT_C: u8 = 29;
const BIT_Z: u8 = 30;
const BIT_N: u8 = 31;
const BIT_I: u8 = 7;
const BIT_F: u8 = 6;
const BIT_T: u8 = 5;
const NOP: u32 = 3785359360;
const BX_27_4: u32 = 0b0001_0010_1111_1111_1111_0001;
const MASK_C: u32 = 0b11011111;
const MASK_F: u32 = 0xff << 24;
const MASK_X: u32 = 0xff << 8;

pub struct Cpu {
    regs: [u32; 37],
    decode_stage: u32,
    execute_stage: u32,
    debug: bool
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Mode {
    User,
    Svc,
    Irq,
    Fiq,
    Und,
    Abt,
    Sys
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum State {
    Arm,
    Thumb
}

impl Default for Cpu {
    fn default() -> Self {
        Cpu::new()
    }
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            regs: [0; 37],
            decode_stage: NOP,
            execute_stage: NOP,
            debug: false
        }
    }

    pub fn reset(&mut self) {
        self.regs[REG_SPSR_SVC] = self.regs[REG_CPSR];
        self.regs[24] = self.regs[15];
        self.regs[REG_CPSR] = 0b11010011;
        self.regs[15] = 0;
    }

    pub fn step(&mut self, ram: &mut Mem) {
        let state = self.get_state();
        let mode = self.get_mode();

        let instruction = self.execute_stage;
        self.execute_stage = self.decode_stage;
        self.decode_stage = ram.get_word(self.regs[15] as usize).little_endian();

        if instruction == NOP {
            self.regs[15] += match state { State::Arm => 4, State::Thumb => 2 };
            if self.debug {
                println!("Executed NOP");
            }
            return;
        }

        if instruction == 0 {    // REMOVE THIS AFTER DEBUGGING -------------------------------------------------------------------------------------------------
            return;
        }

        if let State::Arm = state {
            let sets_flags = 1 == instruction >> 20 & 1;
            let immediate = 1 == instruction >> 25 & 1;
            let opcode = instruction >> 21 & 0b111_1111; // Bits 21 through 27

            let rn_index = self.get_register_index(mode, instruction >> 16 & 0b1111);
            let rn = self.regs[rn_index];
            let rd_index = self.get_register_index(mode, instruction >> 12 & 0b1111);
            let rs = self.regs[self.get_register_index(mode, instruction >> 8 & 0b1111)];
            let rm = self.regs[self.get_register_index(mode, instruction & 0b1111)];

            let mut shifter_carry = self.get_status_bit(BIT_C);

            let operand2 = if immediate {
                let rotate_amount = (instruction >> 8 & 0b1111) * 2;
                (instruction & 0xff).rotate_right(rotate_amount)
            } else {
                let rm = if 1 == instruction >> 4 & 1 && instruction & 0b1111 == 0b1111 { rm + 4 } else { rm }; // PC is +4 if there is a register-specified shift
                let shift_amount = if 1 == instruction >> 4 & 1 { rs & 0xff } else { instruction >> 7 & 0b11111 };
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
                                if self.get_status_bit(BIT_C) { rm >> 1 | 1 << 31 } else { rm >> 1}
                            } else if shift_amount == 32 {
                                shifter_carry = 1 == rm >> 31;
                                rm
                            } else {
                                shifter_carry = 1 == rm >> (shift_amount % 32 - 1) & 1;
                                rm.rotate_right(shift_amount % 32)
                            }
                        }
                        _ => panic!()
                    }
                }
            };

            let mut signed_result = 0; // Decides sticky overflow flag
            let mut c = shifter_carry; // Carry flag

            if self.should_execute_arm(instruction) {
                let mut debug_string = "Unrecognized opcode";
                if (instruction << 4) >> 8 == BX_27_4 { // Branch and Exchange
                    debug_string = "BX";
                    self.regs[15] = rm % 2;
                    self.decode_stage = NOP;
                    self.execute_stage = NOP;
                    self.set_thumb_bit(1 == rm & 1);
                } else if opcode >> 3 == 0 && instruction >> 4 & 0b1111 == 0b1001 { // Multiplication
                    if opcode >> 2 & 1 == 1 { // Long Multiply
                        let signed = opcode >> 1 & 1 == 1;
                        let mut result = (rs as i32 as i64 * rm as i32 as i64) as u64;
                        if opcode & 1 == 1 { // Accumulate
                            debug_string = if signed { "SMLAL" } else { "UMLAL" };
                            result = result.wrapping_add((rn as u64) << 32 | self.regs[rd_index] as u64);
                        } else {
                            debug_string = if signed { "SMULL" } else { "UMULL" };
                        }
                        self.regs[rn_index] = (result >> 32) as u32;
                        self.regs[rd_index] = result as u32;
                        if sets_flags {
                            self.set_status_bit(BIT_N, 1 == result >> 63);
                            self.set_status_bit(BIT_Z, result == 0);
                        }
                    } else { // Normal Multiply
                        debug_string = "MUL";
                        let mut result = rs.wrapping_mul(rm);
                        if opcode & 1 == 1 { // Accumulate
                            debug_string = "MLA";
                            result = result.wrapping_add(self.regs[rd_index]);
                        }
                        self.regs[rn_index] = result;
                        if sets_flags {
                            self.set_status_bit(BIT_N, 1 == result >> 31);
                            self.set_status_bit(BIT_Z, result == 0);
                        }
                    }
                } else if opcode >> 4 == 0 && instruction >> 5 & 0b11 != 0 && instruction >> 4 & 0b1001 == 0b1001 { // Halfword
                    let write_back = instruction >> 21 & 1 == 1;
                    let up = instruction >> 23 & 1 == 1;
                    let pre_index = instruction >> 24 & 1 == 1;
                    let sh = instruction >> 5 & 0b11;

                    let offset = if instruction >> 22 & 1 == 1 { // Immediate offset
                        (instruction >> 4 & 0xf0) | (instruction & 0b1111)
                    } else { // Register offset
                        rm
                    };

                    let offset_address = if up { rn + offset } else { rn - offset };
                    let memory_address = if pre_index { offset_address } else { rn };

                    if instruction >> 20 & 1 == 1 { // Load
                        match sh {
                            0b01 => {
                                debug_string = "LDRH";
                                self.regs[rd_index] = ram.get_halfword(memory_address as usize).little_endian() as u32;
                            }
                            0b10 => {
                                debug_string = "LDRSB";
                                self.regs[rd_index] = ram.get_byte(memory_address as usize) as i8 as u32;
                            }
                            0b11 => {
                                debug_string = "LDRSH";
                                self.regs[rd_index] = ram.get_halfword(memory_address as usize).little_endian() as i16 as u32;
                            }
                            _ => panic!()
                        }
                        if rd_index == 15 {
                            self.decode_stage = NOP;
                            self.execute_stage = NOP;
                        }
                    } else { // Store
                        debug_string = "STRH";
                        let data = (if rd_index == 15 { self.regs[rd_index] + 4 } else { self.regs[rd_index] } & 0xffff) as u16;
                        ram.set_halfword(memory_address as usize, HalfWord::from_u16_le(data));
                    }
                    if write_back {
                        self.regs[rn_index] = offset_address;
                    }
                } else if opcode >> 5 == 0 && opcode & 0b1100 == 0b1000 && !sets_flags { // Modifying PSR
                    let psr_index = if 1 == opcode >> 1 & 1 { self.get_psr_index(mode) } else { REG_CPSR };
                    if 1 == opcode & 1 { // MSR
                        debug_string = "MSR";
                        let control_bits = 1 == instruction >> 16 & 1;
                        let extension_bits = 1 == instruction >> 17 & 1;
                        let flags_bits = 1 == instruction >> 19 & 1;
                        let mut value = self.regs[psr_index];

                        if control_bits && mode != Mode::User { value = (value & !MASK_C) | (operand2 & MASK_C) };
                        if extension_bits { value = (value & !MASK_X) | (operand2 & MASK_X) };
                        if flags_bits { value = (value & !MASK_F) | (operand2 & MASK_F) };

                        self.regs[psr_index] = value;
                        if self.get_mode() != mode && self.debug { println!("CHANGED MODE FROM {:?} to {:?}", mode, self.get_mode()) }
                    } else { // MRS
                        debug_string = "MRS";
                        self.regs[rd_index] = self.regs[psr_index];
                    }
                } else if opcode >> 5 == 0 { // Data Processing
                    let mut write_back = true;
                    let result = match opcode & 0b1111 {
                        0b0000 => { // AND
                            debug_string = "AND";
                            rn & operand2
                        }
                        0b0001 => { // EOR
                            debug_string = "EOR";
                            rn ^ operand2
                        }
                        0b0010 => { // SUB
                            debug_string = "SUB";
                            signed_result = rn as i64 - operand2 as i64;
                            c = signed_result >= 0;
                            rn.wrapping_sub(operand2)
                        }
                        0b0011 => { // RSB
                            debug_string = "RSB";
                            signed_result = operand2 as i64 - rn as i64;
                            c = signed_result >= 0;
                            operand2.wrapping_sub(rn)
                        }
                        0b0100 => { // ADD
                            debug_string = "ADD";
                            signed_result = operand2 as i64 + rn as i64;
                            c = signed_result >= 1 << 32;
                            operand2.wrapping_add(rn)
                        }
                        0b0101 => { // ADC
                            debug_string = "ADC";
                            let carry_in = self.get_status_bit(BIT_C);
                            signed_result = operand2 as i64 + rn as i64 + carry_in as i64;
                            c = signed_result >= 1 << 32;
                            operand2.wrapping_add(rn).wrapping_add(carry_in as u32)
                        }
                        0b0110 => { // SBC
                            debug_string = "SBC";
                            let carry_in = self.get_status_bit(BIT_C);
                            signed_result = rn as i64 - operand2 as i64 + carry_in as i64 - 1;
                            c = signed_result >= 0;
                            rn.wrapping_sub(operand2).wrapping_add(carry_in as u32).wrapping_sub(1)
                        }
                        0b0111 => { // RSC
                            debug_string = "RSC";
                            let carry_in = self.get_status_bit(BIT_C);
                            signed_result = operand2 as i64 - rn as i64 + carry_in as i64 - 1;
                            c = signed_result >= 0;
                            operand2.wrapping_sub(rn).wrapping_add(carry_in as u32).wrapping_sub(1)
                        }
                        0b1000 => { // TST
                            debug_string = "TST";
                            write_back = false;
                            rn & operand2
                        }
                        0b1001 => { // TEQ
                            debug_string = "TEQ";
                            write_back = false;
                            rn ^ operand2
                        }
                        0b1010 => { // CMP
                            debug_string = "CMP";
                            write_back = false;
                            signed_result = rn as i64 - operand2 as i64;
                            c = signed_result >= 0;
                            rn.wrapping_sub(operand2)
                        }
                        0b1011 => { // CMN
                            debug_string = "CMN";
                            write_back = false;
                            signed_result = operand2 as i64 + rn as i64;
                            c = signed_result >= 1 << 32;
                            rn.wrapping_add(operand2)
                        }
                        0b1100 => { // ORR
                            debug_string = "ORR";
                            rn | operand2
                        }
                        0b1101 => { // MOV
                            debug_string = "MOV";
                            operand2
                        }
                        0b1110 => { // BIC
                            debug_string = "BIC";
                            rn & !operand2
                        }
                        0b1111 => { // MVN
                            debug_string = "MVN";
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
                        let current_mode_register = self.get_psr_index(mode);
                        self.regs[REG_CPSR] = self.regs[current_mode_register];
                    }
                    if write_back {
                        self.regs[rd_index] = result;
                        if rd_index == 15 {
                            self.decode_stage = NOP;
                            self.execute_stage = NOP;
                        }
                    }
                } else if opcode >> 4 == 0b101 { // Branch / Branch with Link
                    let offset = (instruction << 8) as i32 >> 6;
                    debug_string = "B";
                    if 1 == instruction >> 24 & 1 {
                        debug_string = "BL";
                        self.regs[self.get_register_index(mode, 14)] = self.regs[15] - 4;
                    }
                    self.regs[15] = (self.regs[15] as i32 + offset) as u32;
                    self.decode_stage = NOP;
                    self.execute_stage = NOP;
                } else if opcode >> 5 == 1 { // Load/Store
                    let write_back = instruction >> 21 & 1 == 1;
                    let byte_quantity = instruction >> 22 & 1 == 1;
                    let up = instruction >> 23 & 1 == 1;
                    let pre_index = instruction >> 24 & 1 == 1;

                    let offset = if !immediate {
                        instruction & 0xfff
                    } else {
                        let rm = if 1 == instruction >> 4 & 1 && instruction & 0b1111 == 0b1111 { rm + 4 } else { rm }; // PC is +4 if there is a register-specified shift
                        let shift_amount = if 1 == instruction >> 4 & 1 { rs & 0xff } else { instruction >> 7 & 0b11111 };
                        if 1 == (instruction >> 4) & 1 && shift_amount == 0 {
                            rm
                        } else {
                            match instruction >> 5 & 0b11 {
                                0b00 => { // Logical Shift Left
                                    if shift_amount >= 32 {
                                        0
                                    } else {
                                        rm << shift_amount
                                    }
                                }
                                0b01 => { // Logical Shift Right
                                    if shift_amount == 0 || shift_amount >= 32 {
                                        0
                                    } else {
                                        rm >> shift_amount
                                    }
                                }
                                0b10 => { // Arithmetic Shift Right
                                    if shift_amount == 0 || shift_amount >= 32 {
                                        if 1 == rm >> 31 { !0 } else { 0 }
                                    } else if 1 == rm >> 31 {
                                        rm >> shift_amount | !(!0 >> shift_amount)
                                    } else {
                                        rm >> shift_amount
                                    }
                                }
                                0b11 => { // Rotate Right
                                    if shift_amount == 0 {
                                        if self.get_status_bit(BIT_C) { rm >> 1 | 1 << 31 } else { rm >> 1}
                                    } else if shift_amount == 32 {
                                        rm
                                    } else {
                                        rm.rotate_right(shift_amount % 32)
                                    }
                                }
                                _ => panic!()
                            }
                        }
                    };

                    let offset_address = if up { rn + offset } else { rn - offset };
                    let memory_address = if pre_index { offset_address } else { rn };

                    if instruction >> 20 & 1 == 1 { // Load
                        if byte_quantity {
                            debug_string = "LDRB";
                            self.regs[rd_index] = ram.get_byte(memory_address as usize) as u32;
                        } else {
                            debug_string = "LDR";
                            let word = ram.get_word(memory_address as usize).little_endian();
                            self.regs[rd_index] = word.rotate_right((memory_address % 4) * 8);
                        }
                        if rd_index == 15 {
                            self.decode_stage = NOP;
                            self.execute_stage = NOP;
                        }
                    } else { // Store
                        let data = if rd_index == 15 { self.regs[rd_index] + 4 } else { self.regs[rd_index] };
                        if byte_quantity {
                            debug_string = "STRB";
                            ram.set_byte(memory_address as usize, data as u8);
                        } else {
                            debug_string = "STR";
                            ram.set_word(memory_address as usize, Word::from_u32_le(data));
                        }
                        if memory_address == self.regs[15] - 4 || memory_address == self.regs[15] { // Self-modifying code
                            self.decode_stage = NOP;
                            self.execute_stage = NOP;
                            self.regs[15] -= 8;
                        }
                    }
                    if write_back {
                        self.regs[rn_index] = offset_address;
                    }
                }
                
                if self.debug { println!("Executed {} with op1 = {}, op2 = {}, dest = {}", 
                                        debug_string, rn as i32, operand2 as i32, rd_index as i32); }
            }

            self.regs[15] += match state { State::Arm => 4, State::Thumb => 2 };
        }
    }

    pub fn toggle_debug(&mut self) {
        self.debug = !self.debug;
    }

    #[inline(always)]
    fn set_state(&mut self, state: State) {
        self.set_status_bit(BIT_T, state == State::Thumb);
    }

    #[inline(always)]
    fn set_thumb_bit(&mut self, thumb: bool) {
        self.set_status_bit(BIT_T, thumb);
    }

    #[inline(always)]
    fn get_register_index(&self, mode: Mode, index: u32) -> usize {
        match index {
            8..=12 if mode == Mode::Fiq => index as usize - 8 + 16,
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
    fn get_psr_index(&self, mode: Mode) -> usize {
        match mode {
            Mode::User | Mode::Sys => { REG_CPSR }
            Mode::Svc => { REG_SPSR_SVC }
            Mode::Irq => { REG_SPSR_IRQ }
            Mode::Fiq => { REG_SPSR_FIQ }
            Mode::Und => { REG_SPSR_UND }
            Mode::Abt => { REG_SPSR_ABT }
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
    fn set_mode(&mut self, mode: Mode) {
        let bits = match mode {
            Mode::User => 0b10000,
            Mode::Fiq => 0b10001,
            Mode::Irq => 0b10010,
            Mode::Svc => 0b10011 ,
            Mode::Abt => 0b10111,
            Mode::Und => 0b11011,
            Mode::Sys => 0b11111
        };
        self.regs[REG_CPSR] = (self.regs[REG_CPSR] & !0b11111) | bits;
    }

    #[inline(always)]
    fn get_state(&self) -> State {
        match (self.regs[REG_CPSR] & 0b100000) == 0 {
            true => State::Arm,
            false => State::Thumb
        }
    }
}