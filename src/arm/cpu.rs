use std::{fs::File, io::Write};

use crate::mem::Mem;

use super::common::{HalfWord, Word};
use anyhow::Result;

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
const THUMB_NOP: u32 = 0x46c0;
const BX_27_4: u32 = 0b0001_0010_1111_1111_1111_0001;
const MASK_C: u32 = 0b11011111;
const MASK_F: u32 = 0xff << 24;
const MASK_X: u32 = 0xff << 8;

pub struct Cpu {
    regs: [u32; 37],
    decode_stage: u32,
    execute_stage: u32,
    debug: bool,
    irq_input: bool,
    fiq_input: bool
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Mode {
    User,
    Svc,
    Irq,
    Fiq,
    Und,
    Abt,
    Sys,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum State {
    Arm,
    Thumb,
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
            debug: false,
            irq_input: false,
            fiq_input: false
        }
    }

    pub fn reset(&mut self) {
        self.regs[REG_SPSR_SVC] = self.regs[REG_CPSR];
        self.regs[24] = self.regs[15];
        self.regs[REG_CPSR] = 0b11010011;
        self.regs[15] = 0;
    }

    pub fn step(&mut self, ram: &mut Mem, cycle: usize) -> Option<()> {
        let state = self.get_state();
        let mode = self.get_mode();

        let instruction = self.execute_stage;
        self.execute_stage = self.decode_stage;
        self.decode_stage = ram.get_word(self.regs[15] as usize).little_endian();

        let pc = if self.regs[15] >= 8 {self.regs[15] - if let State::Arm = state { 8 } else { 4 }} else { 0 };

        // static mut frame_count: u8 = 0;
        // if pc == 0x2b34 {
        //     unsafe {frame_count += 1};
        //     println!("Draw #{} at cycle {}", unsafe {frame_count}, cycle);
        //     if unsafe {frame_count == 6} {
        //         return None;
        //     }
        // }
        self.irq_input = ram.get_byte(0x4000202) != 0 && ram.get_byte(0x4000208) & 1 == 1;

        if self.fiq_input && !self.get_status_bit(BIT_F) || self.irq_input && !self.get_status_bit(BIT_I) {
            let mode = if self.fiq_input { Mode::Fiq } else { Mode::Irq };
            self.regs[self.get_register_index(mode, 14)] = pc + 4;
            self.regs[self.get_psr_index(mode)] = self.regs[REG_CPSR];
            self.set_mode(mode);
            self.set_state(State::Arm);
            self.decode_stage = NOP;
            self.execute_stage = NOP;
            self.regs[15] = if self.fiq_input { 0x1C } else { 0x18 };
            self.set_status_bit(BIT_I, true);
            if self.fiq_input {
                self.set_status_bit(BIT_F, true);
                if self.debug {
                    println!("FIQ triggered");
                }
            } else if self.debug {
                println!("IRQ triggered");
            }
            return Some(());
        }

        if instruction == NOP || instruction == THUMB_NOP {
            self.regs[15] += match state {
                State::Arm => 4,
                State::Thumb => 2,
            };
            if self.debug {
                println!("Executed NOP");
            }
            return Some(());
        }

        if instruction == 0 {
            // REMOVE THIS AFTER DEBUGGING -------------------------------------------------------------------------------------------------
            return None;
        }

        if let State::Arm = state {
            let mut branching = false;

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
                let rm = if 1 == instruction >> 4 & 1 && instruction & 0b1111 == 0b1111 {
                    rm + 4
                } else {
                    rm
                }; // PC is +4 if there is a register-specified shift
                let shift_amount = if 1 == instruction >> 4 & 1 {
                    rs & 0xff
                } else {
                    instruction >> 7 & 0b11111
                };
                if 1 == (instruction >> 4) & 1 && shift_amount == 0 {
                    rm
                } else {
                    match instruction >> 5 & 0b11 {
                        0b00 => {
                            // Logical Shift Left
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
                        0b01 => {
                            // Logical Shift Right
                            if shift_amount == 0 || shift_amount == 32 {
                                shifter_carry = 1 == rm >> 31;
                                0
                            } else if shift_amount > 32 {
                                shifter_carry = false;
                                0
                            } else {
                                shifter_carry = 1 == rm >> (shift_amount - 1) & 1;
                                rm >> shift_amount
                            }
                        }
                        0b10 => {
                            // Arithmetic Shift Right
                            if shift_amount == 0 || shift_amount >= 32 {
                                shifter_carry = 1 == rm >> 31;
                                if 1 == rm >> 31 {
                                    !0
                                } else {
                                    0
                                }
                            } else {
                                shifter_carry = 1 == rm >> (shift_amount - 1) & 1;
                                if 1 == rm >> 31 {
                                    rm >> shift_amount | !(!0 >> shift_amount)
                                } else {
                                    rm >> shift_amount
                                }
                            }
                        }
                        0b11 => {
                            // Rotate Right
                            if shift_amount == 0 {
                                shifter_carry = 1 == rm & 1;
                                if self.get_status_bit(BIT_C) {
                                    rm >> 1 | 1 << 31
                                } else {
                                    rm >> 1
                                }
                            } else if shift_amount == 32 {
                                shifter_carry = 1 == rm >> 31;
                                rm
                            } else {
                                shifter_carry = if shift_amount % 32 == 0 { false }
                                    else {1 == rm >> (shift_amount % 32 - 1) & 1};
                                rm.rotate_right(shift_amount % 32)
                            }
                        }
                        _ => panic!(),
                    }
                }
            };

            let mut signed_result = 0; // Decides sticky overflow flag
            let mut c = shifter_carry; // Carry flag

            if self.should_execute_arm(instruction) {
                let mut debug_string = "Unrecognized Opcode";
                if opcode >> 4 == 3 && instruction >> 4 & 1 == 1 {
                    // Undefined instruction
                    debug_string = "UND";
                    self.regs[REG_SPSR_UND] = self.regs[REG_CPSR];
                    self.regs[self.get_register_index(Mode::Und, 14)] = self.regs[15] - 4;
                    self.set_mode(Mode::Und);
                    if self.debug {
                        println!("Changed mode from {:?} to Und", mode);
                    }
                    branching = true;
                    self.regs[15] = 4;
                    self.set_status_bit(BIT_I, true);
                    self.decode_stage = NOP;
                    self.execute_stage = NOP;
                } else if (instruction << 4) >> 8 == BX_27_4 {
                    // Branch and Exchange
                    debug_string = "BX";
                    self.regs[15] = (rm >> 1) << 1;
                    branching = true;
                    self.decode_stage = NOP;
                    self.execute_stage = NOP;
                    self.set_thumb_bit(1 == rm & 1);
                } else if opcode >> 3 == 0 && instruction >> 4 & 0b1111 == 0b1001 {
                    // Multiplication
                    if opcode >> 2 & 1 == 1 {
                        // Long Multiply
                        let signed = opcode >> 1 & 1 == 1;
                        let mut result = (rs as i32 as i64 * rm as i32 as i64) as u64;
                        if opcode & 1 == 1 {
                            // Accumulate
                            debug_string = if signed { "SMLAL" } else { "UMLAL" };
                            result =
                                result.wrapping_add((rn as u64) << 32 | self.regs[rd_index] as u64);
                        } else {
                            debug_string = if signed { "SMULL" } else { "UMULL" };
                        }
                        self.regs[rn_index] = (result >> 32) as u32;
                        self.regs[rd_index] = result as u32;
                        if sets_flags {
                            self.set_status_bit(BIT_N, 1 == result >> 63);
                            self.set_status_bit(BIT_Z, result == 0);
                        }
                    } else {
                        // Normal Multiply
                        debug_string = "MUL";
                        let mut result = rs.wrapping_mul(rm);
                        if opcode & 1 == 1 {
                            // Accumulate
                            debug_string = "MLA";
                            result = result.wrapping_add(self.regs[rd_index]);
                        }
                        self.regs[rn_index] = result;
                        if sets_flags {
                            self.set_status_bit(BIT_N, 1 == result >> 31);
                            self.set_status_bit(BIT_Z, result == 0);
                        }
                    }
                } else if opcode >> 4 == 0
                    && instruction >> 5 & 0b11 != 0
                    && instruction >> 4 & 0b1001 == 0b1001
                {
                    // Halfword
                    let write_back = instruction >> 21 & 1 == 1;
                    let up = instruction >> 23 & 1 == 1;
                    let pre_index = instruction >> 24 & 1 == 1;
                    let sh = instruction >> 5 & 0b11;

                    let offset = if instruction >> 22 & 1 == 1 {
                        // Immediate offset
                        (instruction >> 4 & 0xf0) | (instruction & 0b1111)
                    } else {
                        // Register offset
                        rm
                    };

                    let offset_address = if up { rn + offset } else { rn - offset };
                    let memory_address = if pre_index { offset_address } else { rn };

                    if instruction >> 20 & 1 == 1 {
                        // Load
                        match sh {
                            0b01 => {
                                debug_string = "LDRH";
                                self.regs[rd_index] =
                                    ram.get_halfword(memory_address as usize).little_endian()
                                        as u32;
                            }
                            0b10 => {
                                debug_string = "LDRSB";
                                self.regs[rd_index] =
                                    ram.get_byte(memory_address as usize) as i8 as u32;
                            }
                            0b11 => {
                                debug_string = "LDRSH";
                                self.regs[rd_index] = ram
                                    .get_halfword(memory_address as usize)
                                    .little_endian()
                                    as i16
                                    as u32;
                            }
                            _ => panic!(),
                        }
                        if rd_index == 15 {
                            branching = true;
                            self.decode_stage = NOP;
                            self.execute_stage = NOP;
                        }
                    } else {
                        // Store
                        debug_string = "STRH";
                        let data = (if rd_index == 15 {
                            self.regs[rd_index] + 4
                        } else {
                            self.regs[rd_index]
                        } & 0xffff) as u16;
                        ram.set_halfword(memory_address as usize, HalfWord::from_u16_le(data));
                    }
                    if write_back || !pre_index {
                        self.regs[rn_index] = offset_address;
                    }
                } else if opcode >> 2 == 2 && instruction >> 4 & 0b1111 == 0b1001 {
                    // Atomic Swap
                    let byte = 1 == instruction >> 22 & 1;
                    if byte {
                        // Byte swap
                        debug_string = "SWPB";
                        self.regs[rd_index] = ram.get_byte(rn as usize) as u32;
                        ram.set_byte(rn as usize, (rm & 0xff) as u8);
                    } else {
                        // Word swap
                        debug_string = "SWP";
                        self.regs[rd_index] = ram.get_word(rn as usize).little_endian();
                        ram.set_word(rn as usize, Word::from_u32_le(rm));
                    }
                } else if opcode >> 5 == 0 && opcode & 0b1100 == 0b1000 && !sets_flags {
                    // Modifying PSR
                    let psr_index = if 1 == opcode >> 1 & 1 {
                        self.get_psr_index(mode)
                    } else {
                        REG_CPSR
                    };
                    if 1 == opcode & 1 {
                        // MSR
                        debug_string = "MSR";
                        let control_bits = 1 == instruction >> 16 & 1;
                        let extension_bits = 1 == instruction >> 17 & 1;
                        let flags_bits = 1 == instruction >> 19 & 1;
                        let mut value = self.regs[psr_index];

                        if control_bits && mode != Mode::User {
                            value = (value & !MASK_C) | (operand2 & MASK_C)
                        };
                        if extension_bits {
                            value = (value & !MASK_X) | (operand2 & MASK_X)
                        };
                        if flags_bits {
                            value = (value & !MASK_F) | (operand2 & MASK_F)
                        };

                        self.regs[psr_index] = value;
                        if self.get_mode() != mode && self.debug {
                            println!("CHANGED MODE FROM {:?} to {:?}", mode, self.get_mode())
                        }
                    } else {
                        // MRS
                        debug_string = "MRS";
                        self.regs[rd_index] = self.regs[psr_index];
                    }
                } else if opcode >> 5 == 0 {
                    // Data Processing
                    let mut write_back = true;
                    let result = match opcode & 0b1111 {
                        0b0000 => {
                            // AND
                            debug_string = "AND";
                            rn & operand2
                        }
                        0b0001 => {
                            // EOR
                            debug_string = "EOR";
                            rn ^ operand2
                        }
                        0b0010 => {
                            // SUB
                            debug_string = "SUB";
                            signed_result = rn as i32 as i64 - operand2 as i32 as i64;
                            c = signed_result >= 0;
                            rn.wrapping_sub(operand2)
                        }
                        0b0011 => {
                            // RSB
                            debug_string = "RSB";
                            signed_result = operand2 as i32 as i64 - rn as i32 as i64;
                            c = signed_result >= 0;
                            operand2.wrapping_sub(rn)
                        }
                        0b0100 => {
                            // ADD
                            debug_string = "ADD";
                            signed_result = operand2 as i32 as i64 + rn as i32 as i64;
                            c = signed_result >= 1 << 32;
                            operand2.wrapping_add(rn)
                        }
                        0b0101 => {
                            // ADC
                            debug_string = "ADC";
                            let carry_in = self.get_status_bit(BIT_C);
                            signed_result = operand2 as i32 as i64 + rn as i32 as i64 + carry_in as i64;
                            c = signed_result >= 1 << 32;
                            operand2.wrapping_add(rn).wrapping_add(carry_in as u32)
                        }
                        0b0110 => {
                            // SBC
                            debug_string = "SBC";
                            let carry_in = self.get_status_bit(BIT_C);
                            signed_result = rn as i32 as i64 - operand2 as i32 as i64 + carry_in as i64 - 1;
                            c = signed_result >= 0;
                            rn.wrapping_sub(operand2)
                                .wrapping_add(carry_in as u32)
                                .wrapping_sub(1)
                        }
                        0b0111 => {
                            // RSC
                            debug_string = "RSC";
                            let carry_in = self.get_status_bit(BIT_C);
                            signed_result = operand2 as i32 as i64 - rn as i32 as i64 + carry_in as i64 - 1;
                            c = signed_result >= 0;
                            operand2
                                .wrapping_sub(rn)
                                .wrapping_add(carry_in as u32)
                                .wrapping_sub(1)
                        }
                        0b1000 => {
                            // TST
                            debug_string = "TST";
                            write_back = false;
                            rn & operand2
                        }
                        0b1001 => {
                            // TEQ
                            debug_string = "TEQ";
                            write_back = false;
                            rn ^ operand2
                        }
                        0b1010 => {
                            // CMP
                            debug_string = "CMP";
                            write_back = false;
                            signed_result = rn as i32 as i64 - operand2 as i32 as i64;
                            c = signed_result >= 0;
                            rn.wrapping_sub(operand2)
                        }
                        0b1011 => {
                            // CMN
                            debug_string = "CMN";
                            write_back = false;
                            signed_result = operand2 as i32 as i64 + rn as i32 as i64;
                            c = signed_result >= 1 << 32;
                            rn.wrapping_add(operand2)
                        }
                        0b1100 => {
                            // ORR
                            debug_string = "ORR";
                            rn | operand2
                        }
                        0b1101 => {
                            // MOV
                            debug_string = "MOV";
                            operand2
                        }
                        0b1110 => {
                            // BIC
                            debug_string = "BIC";
                            rn & !operand2
                        }
                        0b1111 => {
                            // MVN
                            debug_string = "MVN";
                            !operand2
                        }
                        _ => panic!(),
                    };
                    if sets_flags && rd_index != 15 {
                        self.set_status_bit(BIT_C, c);
                        self.set_status_bit(BIT_N, 1 == result >> 31);
                        self.set_status_bit(BIT_Z, result == 0);
                        if signed_result >= 1 << 31 || signed_result < -(1 << 31) {
                            self.set_status_bit(BIT_V, true);
                        }
                    } else if sets_flags {
                        // Transfer the SPSR
                        let current_mode_register = self.get_psr_index(mode);
                        self.regs[REG_CPSR] = self.regs[current_mode_register];
                    }
                    if write_back {
                        self.regs[rd_index] = result;
                        if rd_index == 15 {
                            self.decode_stage = NOP;
                            self.execute_stage = NOP;
                            branching = true;
                        }
                    }
                } else if opcode >> 4 == 0b101 {
                    // Branch / Branch with Link
                    let offset = (instruction << 8) as i32 >> 6;
                    debug_string = "B";
                    if 1 == instruction >> 24 & 1 {
                        debug_string = "BL";
                        self.regs[self.get_register_index(mode, 14)] = self.regs[15] - 4;
                    }
                    self.regs[15] = (self.regs[15] as i32 + offset) as u32;
                    branching = true;
                    self.decode_stage = NOP;
                    self.execute_stage = NOP;
                } else if opcode >> 5 == 1 {
                    // Load/Store
                    let byte_quantity = instruction >> 22 & 1 == 1;
                    let up = instruction >> 23 & 1 == 1;
                    let pre_index = instruction >> 24 & 1 == 1;
                    let write_back = instruction >> 21 & 1 == 1;

                    let offset = if !immediate {
                        instruction & 0xfff
                    } else {
                        let rm = if 1 == instruction >> 4 & 1 && instruction & 0b1111 == 0b1111 {
                            rm + 4
                        } else {
                            rm
                        }; // PC is +4 if there is a register-specified shift
                        let shift_amount = if 1 == instruction >> 4 & 1 {
                            rs & 0xff
                        } else {
                            instruction >> 7 & 0b11111
                        };
                        if 1 == (instruction >> 4) & 1 && shift_amount == 0 {
                            rm
                        } else {
                            match instruction >> 5 & 0b11 {
                                0b00 => {
                                    // Logical Shift Left
                                    if shift_amount >= 32 {
                                        0
                                    } else {
                                        rm << shift_amount
                                    }
                                }
                                0b01 => {
                                    // Logical Shift Right
                                    if shift_amount == 0 || shift_amount >= 32 {
                                        0
                                    } else {
                                        rm >> shift_amount
                                    }
                                }
                                0b10 => {
                                    // Arithmetic Shift Right
                                    if shift_amount == 0 || shift_amount >= 32 {
                                        if 1 == rm >> 31 {
                                            !0
                                        } else {
                                            0
                                        }
                                    } else if 1 == rm >> 31 {
                                        rm >> shift_amount | !(!0 >> shift_amount)
                                    } else {
                                        rm >> shift_amount
                                    }
                                }
                                0b11 => {
                                    // Rotate Right
                                    if shift_amount == 0 {
                                        if self.get_status_bit(BIT_C) {
                                            rm >> 1 | 1 << 31
                                        } else {
                                            rm >> 1
                                        }
                                    } else if shift_amount == 32 {
                                        rm
                                    } else {
                                        rm.rotate_right(shift_amount % 32)
                                    }
                                }
                                _ => panic!(),
                            }
                        }
                    };

                    let offset_address = if up { rn + offset } else { rn - offset };
                    let memory_address = if pre_index { offset_address } else { rn };

                    if instruction >> 20 & 1 == 1 {
                        // Load
                        if byte_quantity {
                            debug_string = "LDRB";
                            self.regs[rd_index] = ram.get_byte(memory_address as usize) as u32;
                        } else {
                            debug_string = "LDR";
                            let word = ram.get_word(memory_address as usize).little_endian();
                            self.regs[rd_index] = word.rotate_right((memory_address % 4) * 8);
                        }
                        if rd_index == 15 {
                            branching = true;
                            self.decode_stage = NOP;
                            self.execute_stage = NOP;
                        }
                    } else {
                        // Store
                        let data = if rd_index == 15 {
                            self.regs[rd_index] + 4
                        } else {
                            self.regs[rd_index]
                        };
                        if byte_quantity {
                            debug_string = "STRB";
                            ram.set_byte(memory_address as usize, data as u8);
                        } else {
                            debug_string = "STR";
                            ram.set_word(memory_address as usize, Word::from_u32_le(data));
                        }
                        if memory_address == self.regs[15] - 4 || memory_address == self.regs[15] {
                            // Self-modifying code
                            self.decode_stage = NOP;
                            self.execute_stage = NOP;
                            branching = true;
                            self.regs[15] -= 8;
                        }
                    }
                    if write_back || !pre_index {
                        self.regs[rn_index] = offset_address;
                    }
                } else if opcode >> 4 == 4 {
                    // Load Multiple / Store Multiple
                    let up = instruction >> 23 & 1 == 1;
                    let pre_index = instruction >> 24 & 1 == 1;
                    let load_psr = instruction >> 22 & 1 == 1;
                    let write_back = instruction >> 21 & 1 == 1;

                    let mut num_regs = 0;
                    let mut bits = instruction & 0xffff;
                    while bits != 0 {
                        bits = bits & (bits - 1);
                        num_regs += 1;
                    }

                    let mut memory_address = if up {
                        if pre_index {
                            rn + 4
                        } else {
                            rn
                        }
                    } else if pre_index {
                        rn - 4 * num_regs
                    } else {
                        rn - 4 * num_regs + 4
                    };

                    let old_rn = self.regs[rn_index];
                    if write_back {
                        self.regs[rn_index] = if up {
                            rn + num_regs * 4
                        } else {
                            rn - num_regs * 4
                        };
                    }

                    if instruction >> 20 & 1 == 1 {
                        // Load
                        debug_string = "LDM";
                        for bit in 0..16 {
                            if instruction >> bit & 1 == 1 {
                                let index = if load_psr && instruction >> 15 & 1 == 0 {
                                    bit as usize
                                } else {
                                    self.get_register_index(mode, bit)
                                };
                                self.regs[index] =
                                    ram.get_word(memory_address as usize).little_endian();
                                memory_address += 4;
                                if bit == 15 {
                                    branching = true;
                                    self.decode_stage = NOP;
                                    self.execute_stage = NOP;
                                }
                            }
                        }
                        if load_psr && instruction >> 15 & 1 == 1 {
                            self.regs[REG_CPSR] = self.regs[self.get_psr_index(mode)];
                            if self.debug {
                                println!("Changed from {:?} to mode {:?}", mode, self.get_mode());
                            }
                        }
                    } else {
                        // Store
                        debug_string = "STM";
                        let mut first = true;
                        for bit in 0..16 {
                            if instruction >> bit & 1 == 1 {
                                let mut data = if load_psr {
                                    self.regs[bit as usize]
                                } else {
                                    self.regs[self.get_register_index(mode, bit)]
                                };
                                if bit == 15 {
                                    data += 4
                                };
                                if first && self.get_register_index(mode, bit) == rn_index {
                                    data = old_rn;
                                }
                                ram.set_word(memory_address as usize, Word::from_u32_le(data));
                                memory_address += 4;
                                first = false;
                            }
                        }
                    }
                } else if opcode >> 3 == 15 {
                    // SWI
                    debug_string = "SWI";
                    self.regs[REG_SPSR_SVC] = self.regs[REG_CPSR];
                    self.regs[self.get_register_index(Mode::Svc, 14)] = self.regs[15] - 4;
                    self.set_mode(Mode::Svc);
                    if self.debug {
                        println!("Changed mode from {:?} to Svc", mode);
                    }
                    branching = true;
                    self.regs[15] = 8;
                    self.decode_stage = NOP;
                    self.execute_stage = NOP;
                    self.set_status_bit(BIT_I, true);
                }

                if self.debug {
                    println!(
                        "Executed {} with op1 = {}, op2 = {}, dest = {} at PC = 0x{:x}",
                        debug_string, rn as i32, operand2 as i32, rd_index as i32, pc
                    );
                }
            }
            if !branching {
                self.regs[15] += 4;
            }
        } else {
            let instruction = instruction & 0xffff;
            let mut branching = false;
            let opcode = instruction >> 11;

            let mut rd_index = (instruction & 0b111) as usize;
            let mut rd = self.regs[rd_index];
            let rs_index = (instruction >> 3 & 0b111) as usize;
            let rs = self.regs[rs_index];

            let mut debug_string = "Unrecognized Opcode";
            let mut executed = true;

            if opcode == 3 {
                // Add/Sub
                let immediate = 1 == instruction >> 10 & 1;
                let sub = 1 == instruction >> 9 & 1;
                let value = instruction >> 6 & 0b111;
                let value = if immediate {
                    value
                } else {
                    self.regs[value as usize]
                };
                let c;
                let signed_result;
                let result = if sub {
                    debug_string = "SUB";
                    signed_result = rs as i32 as i64 - value as i32 as i64;
                    c = signed_result >= 0;
                    rs.wrapping_sub(value)
                } else {
                    debug_string = "ADD";
                    signed_result = rs as i32 as i64 + value as i32 as i64;
                    c = signed_result >= 1 << 32;
                    rs.wrapping_add(value)
                };
                self.regs[rd_index] = result;

                self.set_status_bit(BIT_C, c);
                self.set_status_bit(BIT_N, 1 == result >> 31);
                self.set_status_bit(BIT_Z, result == 0);
                if signed_result >= 1 << 31 || signed_result < -(1 << 31) {
                    self.set_status_bit(BIT_V, true);
                }
            } else if opcode >> 2 == 0 {
                // Shifted Move
                let mut shifter_carry = self.get_status_bit(BIT_C);
                let shift_amount = instruction >> 6 & 0b11111;
                let result = match opcode {
                    0 => {
                        // Logical Shift Left
                        debug_string = "LSL";
                        if shift_amount == 0 {
                            rs
                        } else {
                            shifter_carry = 1 == rs >> (32 - shift_amount) & 1;
                            rs << shift_amount
                        }
                    }
                    1 => {
                        // Logical Shift Right
                        debug_string = "LSR";
                        if shift_amount == 0 {
                            shifter_carry = 1 == rs >> 31;
                            0
                        } else {
                            shifter_carry = 1 == rs >> (shift_amount - 1) & 1;
                            rs >> shift_amount
                        }
                    }
                    2 => {
                        // Arithmetic Shift Right
                        debug_string = "ASR";
                        if shift_amount == 0 {
                            shifter_carry = 1 == rs >> 31;
                            if 1 == rs >> 31 {
                                !0
                            } else {
                                0
                            }
                        } else {
                            shifter_carry = 1 == rs >> (shift_amount - 1) & 1;
                            if 1 == rs >> 31 {
                                rs >> shift_amount | !(!0 >> shift_amount)
                            } else {
                                rs >> shift_amount
                            }
                        }
                    }
                    _ => panic!(),
                };
                self.regs[rd_index] = result;
                self.set_status_bit(BIT_C, shifter_carry);
                self.set_status_bit(BIT_N, 1 == result >> 31);
                self.set_status_bit(BIT_Z, result == 0);
            } else if opcode >> 2 == 1 {
                // Move / Compare // Add / Subtract Immediate
                let immediate = instruction & 0xff;
                let mut c = self.get_status_bit(BIT_C);
                let mut signed_result = 0;
                let mut write_back = true;
                rd_index = (instruction >> 8 & 0b111) as usize;
                rd = self.regs[rd_index];
                let result = match opcode & 0b11 {
                    0 => {
                        // MOV
                        debug_string = "MOV";
                        immediate
                    }
                    1 => {
                        // CMP
                        debug_string = "CMP";
                        signed_result = rd as i32 as i64 - immediate as i32 as i64;
                        c = signed_result >= 0;
                        write_back = false;
                        rd.wrapping_sub(immediate)
                    }
                    2 => {
                        // ADD
                        debug_string = "ADD";
                        signed_result = rd as i32 as i64 + immediate as i32 as i64;
                        c = signed_result >= 1 << 32;
                        rd.wrapping_add(immediate)
                    }
                    3 => {
                        // SUB
                        debug_string = "SUB";
                        signed_result = rd as i32 as i64 - immediate as i32 as i64;
                        c = signed_result >= 0;
                        rd.wrapping_sub(immediate)
                    }
                    _ => panic!(),
                };
                self.set_status_bit(BIT_C, c);
                self.set_status_bit(BIT_N, 1 == result >> 31);
                self.set_status_bit(BIT_Z, result == 0);
                if signed_result >= 1 << 31 || signed_result < -(1 << 31) {
                    self.set_status_bit(BIT_V, true);
                }
                if write_back {
                    self.regs[rd_index] = result;
                }
            } else if opcode == 8 && instruction >> 10 & 1 == 0 {
                // ALU
                let mut c = self.get_status_bit(BIT_C);
                let mut signed_result = 0;
                let mut write_back = true;
                let result = match instruction >> 6 & 0b1111 {
                    0 => {
                        // AND
                        debug_string = "AND";
                        rd & rs
                    }
                    1 => {
                        // EOR
                        debug_string = "EOR";
                        rd ^ rs
                    }
                    2 => {
                        // LSL
                        debug_string = "LSL";
                        if rs == 0 {
                            rd
                        } else if rs == 32 {
                            c = rd & 1 == 1;
                            0
                        } else if rs >= 33 {
                            c = false;
                            0
                        } else {
                            c = 1 == rd >> (32 - rs) & 1;
                            rd << rs
                        }
                    }
                    3 => {
                        // LSR
                        debug_string = "LSR";
                        if rs == 0 {
                            rd
                        } else if rs == 32 {
                            c = 1 == rd >> 31;
                            0
                        } else if rs >= 33 {
                            c = false;
                            0
                        } else {
                            c = 1 == rd >> (rs - 1) & 1;
                            rd >> rs
                        }
                    }
                    4 => {
                        // ASR
                        debug_string = "ASR";
                        if rs == 0 {
                            rd
                        } else if rs >= 32 {
                            c = 1 == rd >> 31;
                            if 1 == rd >> 31 {
                                !0
                            } else {
                                0
                            }
                        } else {
                            c = 1 == rd >> (rs - 1) & 1;
                            ((rd as i32) >> rs) as u32
                        }
                    }
                    5 => {
                        // ADC
                        debug_string = "ADC";
                        signed_result = rd as i32 as i64 + rs as i32 as i64 + c as i64;
                        c = signed_result >= 1 << 32;
                        rd.wrapping_add(rs).wrapping_add(c as u32)
                    }
                    6 => {
                        // SBC
                        debug_string = "SBC";
                        signed_result = rd as i32 as i64 - rs as i32 as i64 - !c as i64;
                        c = signed_result >= 0;
                        rd.wrapping_sub(rs).wrapping_sub(!c as u32)
                    }
                    7 => {
                        // ROR
                        debug_string = "ROR";
                        if rs == 0 {
                            rd
                        } else if rs == 32 {
                            c = 1 == rd >> 31;
                            rd
                        } else {
                            c = 1 == rd >> (rs % 32 - 1) & 1;
                            rd.rotate_right(rs % 32)
                        }
                    }
                    8 => {
                        // TST
                        debug_string = "TST";
                        write_back = false;
                        rd & rs
                    }
                    9 => {
                        // NEG
                        debug_string = "NEG";
                        signed_result = 0i64 - rd as i32 as i64;
                        c = signed_result >= 0;
                        0u32.wrapping_sub(rs)
                    }
                    10 => {
                        // CMP
                        debug_string = "CMP";
                        signed_result = rd as i32 as i64 - rs as i32 as i64;
                        c = signed_result >= 0;
                        write_back = false;
                        rd.wrapping_sub(rs)
                    }
                    11 => {
                        // CMN
                        debug_string = "CMN";
                        signed_result = rd as i32 as i64 + rs as i32 as i64;
                        c = signed_result >= 1 << 32;
                        write_back = false;
                        rd.wrapping_add(rs)
                    }
                    12 => {
                        // ORR
                        debug_string = "ORR";
                        rd | rs
                    }
                    13 => {
                        // MUL
                        debug_string = "MUL";
                        rd.wrapping_mul(rs)
                    }
                    14 => {
                        // BIC
                        debug_string = "BIC";
                        rd & !rs
                    }
                    15 => {
                        // MVN
                        debug_string = "MVN";
                        !rs
                    }
                    _ => panic!(),
                };
                self.set_status_bit(BIT_C, c);
                self.set_status_bit(BIT_N, 1 == result >> 31);
                self.set_status_bit(BIT_Z, result == 0);
                if signed_result >= 1 << 31 || signed_result < -(1 << 31) {
                    self.set_status_bit(BIT_V, true);
                }
                if write_back {
                    self.regs[rd_index] = result;
                }
            } else if opcode == 8 {
                // Hi registers/ BX
                let h2 = instruction >> 6 & 1 == 1;
                let h1 = instruction >> 7 & 1 == 1;
                rd_index = if h1 {
                    self.get_register_index(mode, (instruction & 0b111) + 8)
                } else {
                    rd_index
                };
                let rs_index = if h2 {
                    self.get_register_index(mode, (instruction >> 3 & 0b111) + 8)
                } else {
                    rs_index
                };
                match instruction >> 8 & 0b11 {
                    0 => {
                        // ADD
                        debug_string = "ADD";
                        self.regs[rd_index] += self.regs[rs_index];
                    }
                    1 => {
                        // CMP
                        debug_string = "CMP";
                        let signed_result = self.regs[rd_index] as i32 as i64 - self.regs[rs_index] as i32 as i64;
                        let result = self.regs[rd_index].wrapping_sub(self.regs[rs_index]);
                        self.set_status_bit(BIT_C, signed_result >= 0);
                        self.set_status_bit(BIT_N, result >> 31 == 1);
                        self.set_status_bit(BIT_Z, result == 0);
                        if signed_result >= 1 << 31 || signed_result < -(1 << 31) {
                            self.set_status_bit(BIT_V, true);
                        }
                    }
                    2 => {
                        // MOV
                        debug_string = "MOV";
                        self.regs[rd_index] = self.regs[rs_index];
                    }
                    3 => {
                        // BX
                        debug_string = "BX";
                        self.regs[15] = (self.regs[rs_index] >> 1) << 1;
                        branching = true;
                        self.decode_stage = NOP;
                        self.execute_stage = NOP;
                        self.set_thumb_bit(1 == self.regs[rs_index] & 1);
                    }
                    _ => panic!(),
                }
            } else if opcode == 9 {
                // PC-relative load
                debug_string = "LDR";
                rd_index = (instruction >> 8 & 0b111) as usize;
                let address = ((instruction & 0xff) << 2) + ((self.regs[15] >> 2) << 2);
                self.regs[rd_index] = ram.get_word(address as usize).little_endian();
            } else if opcode >> 1 == 5 && instruction >> 9 & 1 == 0 {
                // Load/store with register offset
                let load = instruction >> 11 & 1 == 1;
                let byte = instruction >> 10 & 1 == 1;
                let offset = self.regs[(instruction >> 6 & 0b111) as usize];
                let memory_address = rs.wrapping_add(offset);
                if load {
                    if byte {
                        debug_string = "LDRB";
                        self.regs[rd_index] = ram.get_byte(memory_address as usize) as u32;
                    } else {
                        debug_string = "LDR";
                        let data = ram.get_word(memory_address as usize).little_endian();
                        self.regs[rd_index] = data.rotate_right((memory_address % 4) * 8);
                    }
                } else if byte {
                    debug_string = "STRB";
                    ram.set_byte(memory_address as usize, (rd & 0xff) as u8);
                } else {
                    debug_string = "STR";
                    ram.set_word(memory_address as usize, Word::from_u32_le(rd));
                }
            } else if opcode >> 1 == 5 {
                // Load/store sign-extended byte/halfword
                let offset = self.regs[(instruction >> 6 & 0b111) as usize];
                let memory_address = rs.wrapping_add(offset);
                match instruction >> 10 & 0b11 {
                    0 => {
                        debug_string = "STRH";
                        ram.set_halfword(
                            memory_address as usize,
                            HalfWord::from_u16_le((rd & 0xffff) as u16),
                        );
                    }
                    1 => {
                        debug_string = "LDSB";
                        self.regs[rd_index] = ram.get_byte(memory_address as usize) as i8 as u32;
                    }
                    2 => {
                        debug_string = "LDRH";
                        self.regs[rd_index] =
                            ram.get_halfword(memory_address as usize).little_endian() as u32;
                    }
                    3 => {
                        debug_string = "LDSH";
                        self.regs[rd_index] =
                            ram.get_halfword(memory_address as usize).little_endian() as i16 as u32;
                    }
                    _ => panic!(),
                }
            } else if opcode >> 2 == 3 {
                // Load/Store with immediate offset
                let byte = instruction >> 12 & 1 == 1;
                let mut offset = instruction >> 6 & 0b11111;
                if !byte {
                    offset <<= 2;
                }
                let memory_address = rs.wrapping_add(offset);
                if instruction >> 11 & 1 == 1 {
                    // Load
                    self.regs[rd_index] = if byte {
                        debug_string = "LDRB";
                        ram.get_byte(memory_address as usize) as u32
                    } else {
                        debug_string = "LDR";
                        ram.get_word(memory_address as usize)
                            .little_endian()
                            .rotate_right((memory_address % 4) * 8)
                    };
                } else {
                    // Store
                    if byte {
                        debug_string = "STRB";
                        ram.set_byte(memory_address as usize, (rd & 0xff) as u8);
                    } else {
                        debug_string = "STR";
                        ram.set_word(memory_address as usize, Word::from_u32_le(rd));
                    }
                }
            } else if opcode >> 1 == 8 {
                // Load/store halfword
                let offset = (instruction >> 6 & 0b11111) << 1;
                let memory_address = rs.wrapping_add(offset);
                if instruction >> 11 & 1 == 1 {
                    // Load
                    debug_string = "LDRH";
                    self.regs[rd_index] =
                        ram.get_halfword(memory_address as usize).little_endian() as u32;
                } else {
                    // Store
                    debug_string = "STRH";
                    ram.set_halfword(
                        memory_address as usize,
                        HalfWord::from_u16_le((rd & 0xffff) as u16),
                    );
                }
            } else if opcode >> 1 == 9 {
                // SP-relative load/store
                let offset = (instruction & 0xff) << 2;
                let sp = self.regs[self.get_register_index(mode, 13)];
                rd_index = (instruction >> 8 & 0b111) as usize;
                rd = self.regs[rd_index];
                if instruction >> 11 & 1 == 1 {
                    // Load
                    debug_string = "LDR";
                    self.regs[rd_index] = ram.get_word((sp + offset) as usize).little_endian();
                } else {
                    // Store
                    debug_string = "STR";
                    ram.set_word((sp + offset) as usize, Word::from_u32_le(rd));
                }
            } else if opcode >> 1 == 10 {
                // Load address
                let sp = instruction >> 11 & 1 == 1;
                let offset = (instruction & 0xff) << 2;
                rd_index = (instruction >> 8 & 0b111) as usize;
                debug_string = "LEA";
                self.regs[rd_index] = offset
                    + if sp {
                        self.regs[self.get_register_index(mode, 13)]
                    } else {
                        (self.regs[15] >> 2) << 2
                    };
            } else if opcode >> 1 == 11 && instruction >> 8 & 0b1111 == 0 {
                // Add offset to SP
                debug_string = "ADD_SP";
                let sp_index = self.get_register_index(mode, 13);
                let offset = (instruction & 0b111_1111) << 2;
                if instruction >> 7 & 1 == 1 {
                    self.regs[sp_index] -= offset;
                } else {
                    self.regs[sp_index] += offset;
                }
            } else if opcode >> 1 == 11 {
                // Push/pop registers
                let pop = instruction >> 11 & 1 == 1;
                let pc_lr = instruction >> 8 & 1 == 1;
                let sp_index = self.get_register_index(mode, 13);
                if pop {
                    debug_string = "POP";
                    let mut memory_address = self.regs[sp_index];
                    for bit in 0..8 {
                        if instruction >> bit & 1 == 1 {
                            self.regs[bit] = ram.get_word(memory_address as usize).little_endian();
                            memory_address += 4;
                        }
                    }
                    if pc_lr {
                        self.regs[15] = ram.get_word(memory_address as usize).little_endian();
                        memory_address += 4;
                        self.decode_stage = NOP;
                        self.execute_stage = NOP;
                        branching = true;
                    }
                    self.regs[sp_index] = memory_address;
                } else {
                    debug_string = "PUSH";
                    let mut num_regs = 0;
                    let mut bits = instruction & 0xff;
                    while bits != 0 {
                        bits = bits & (bits - 1);
                        num_regs += 1;
                    }
                    if pc_lr {
                        num_regs += 1
                    };
                    let mut memory_address = self.regs[sp_index] - 4 * num_regs;
                    self.regs[sp_index] = memory_address;
                    for bit in 0..8 {
                        if instruction >> bit & 1 == 1 {
                            ram.set_word(
                                memory_address as usize,
                                Word::from_u32_le(self.regs[bit]),
                            );
                            memory_address += 4;
                        }
                    }
                    if pc_lr {
                        let data = self.regs[self.get_register_index(mode, 14)];
                        ram.set_word(memory_address as usize, Word::from_u32_le(data));
                    }
                }
            } else if opcode >> 1 == 12 {
                // Load multiple/Store multiple
                let load = instruction >> 11 & 1 == 1;
                let rb_index = (instruction >> 8 & 0b111) as usize;
                let mut memory_address = self.regs[rb_index];
                if load {
                    debug_string = "LDMIA";
                    for bit in 0..8 {
                        if instruction >> bit & 1 == 1 {
                            self.regs[bit] = ram.get_word(memory_address as usize).little_endian();
                            memory_address += 4;
                        }
                    }
                    if instruction >> rb_index & 1 == 0 {
                        self.regs[rb_index] = memory_address;
                    }
                } else {
                    debug_string = "STMIA";
                    let mut num_regs = 0;
                    let mut bits = instruction & 0xff;
                    while bits != 0 {
                        bits = bits & (bits - 1);
                        num_regs += 1;
                    }
                    self.regs[rb_index] = memory_address + 4 * num_regs;
                    let old_rb = memory_address;
                    let mut first = true;
                    for bit in 0..8 {
                        if instruction >> bit & 1 == 1 {
                            let mut data = self.regs[bit];
                            if first && bit == rb_index {
                                data = old_rb;
                            }
                            ram.set_word(memory_address as usize, Word::from_u32_le(data));
                            memory_address += 4;
                            first = false;
                        }
                    }
                }
            } else if opcode >> 1 == 13 && instruction >> 8 & 0b11111 == 0b11111 {
                // SWI
                debug_string = "SWI";
                branching = true;
                self.regs[REG_SPSR_SVC] = self.regs[REG_CPSR];
                self.regs[self.get_register_index(Mode::Svc, 14)] = self.regs[15] - 2;
                self.set_mode(Mode::Svc);
                self.set_state(State::Arm);
                if self.debug {
                    println!("Changed state from Thumb to Arm");
                }
                if self.debug {
                    println!("Changed mode from {:?} to Svc", mode);
                }
                self.regs[15] = 8;
                self.decode_stage = NOP;
                self.execute_stage = NOP;
                self.set_status_bit(BIT_I, true);
            } else if opcode >> 1 == 13 {
                // Conditional Branch
                debug_string = "B";
                let cond = instruction >> 8 & 0b1111;
                let c = self.get_status_bit(BIT_C);
                let v = self.get_status_bit(BIT_V);
                let n = self.get_status_bit(BIT_N);
                let z = self.get_status_bit(BIT_Z);
                let branch = match cond {
                    0 => z,
                    1 => !z,
                    2 => c,
                    3 => !c,
                    4 => n,
                    5 => !n,
                    6 => v,
                    7 => !v,
                    8 => c && !z,
                    9 => !c || z,
                    10 => n == v,
                    11 => n != v,
                    12 => !z && n == v,
                    13 => z || n != v,
                    _ => panic!(),
                };
                executed = branch;
                let offset = ((instruction << 24) as i32) >> 23;
                if branch {
                    branching = true;
                    self.decode_stage = NOP;
                    self.execute_stage = NOP;
                    self.regs[15] = (self.regs[15] as i32 + offset) as u32;
                }
            } else if opcode >> 1 == 14 {
                // Branch
                debug_string = "B";
                let offset = ((instruction << 21) as i32) >> 20;
                branching = true;
                self.decode_stage = NOP;
                self.execute_stage = NOP;
                self.regs[15] = (self.regs[15] as i32 + offset) as u32;
            } else if opcode >> 1 == 15 {
                // Long Branch with Link
                debug_string = "BL";
                let h = instruction >> 11 & 1 == 1;
                if h {
                    // part 2 : jump
                    let upper = self.regs[self.get_register_index(mode, 14)];
                    self.regs[self.get_register_index(mode, 14)] = (self.regs[15] - 2) | 1;
                    self.regs[15] = (upper as i32 + (instruction << 1 & 0xfff) as i32) as u32;
                    branching = true;
                    self.decode_stage = NOP;
                    self.execute_stage = NOP;
                } else {
                    // part 1 : load address
                    self.regs[self.get_register_index(mode, 14)] =
                        self.regs[15].wrapping_add((((instruction << 21) as i32) >> 9) as u32);
                }
            }
            if self.debug && executed {
                println!(
                    "Executed THUMB {}, rd ({}) is now 0x{:x} at PC = 0x{:x}",
                    debug_string, rd_index, self.regs[rd_index] as i32, pc
                );
            }
            if !branching {
                self.regs[15] += 2;
            }
        }

        if self.regs[15] % 2 == 1 {
            self.regs[15] = (self.regs[15] >> 1) << 1;
        }

        Some(())
    }

    pub fn toggle_debug(&mut self) {
        self.debug = !self.debug;
    }

    pub fn fiq(&mut self) {
        self.fiq_input = true;
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
            },
            _ => index as usize,
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
            Mode::User | Mode::Sys => REG_CPSR,
            Mode::Svc => REG_SPSR_SVC,
            Mode::Irq => REG_SPSR_IRQ,
            Mode::Fiq => REG_SPSR_FIQ,
            Mode::Und => REG_SPSR_UND,
            Mode::Abt => REG_SPSR_ABT,
        }
    }

    #[inline(always)]
    fn should_execute_arm(&self, instruction: u32) -> bool {
        let v = 1 == self.regs[REG_CPSR] >> 28 & 1;
        let c = 1 == self.regs[REG_CPSR] >> 29 & 1;
        let z = 1 == self.regs[REG_CPSR] >> 30 & 1;
        let n = 1 == self.regs[REG_CPSR] >> 31;

        match instruction >> 28 {
            0b0000 => z,              // EQ
            0b0001 => !z,             // NE
            0b0010 => c,              // CS/HS
            0b0011 => !c,             // CC/LO
            0b0100 => n,              // MI
            0b0101 => !n,             // PL
            0b0110 => v,              // VS
            0b0111 => !v,             // VC
            0b1000 => c && !z,        // HI
            0b1001 => !c || z,        // LS
            0b1010 => n == v,         // GE
            0b1011 => n != v,         // LT
            0b1100 => !z && (n == v), // GT
            0b1101 => z || (n != v),  // LE
            0b1110 => true,
            _ => panic!(),
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
            _ => panic!(),
        }
    }

    #[inline(always)]
    fn set_mode(&mut self, mode: Mode) {
        let bits = match mode {
            Mode::User => 0b10000,
            Mode::Fiq => 0b10001,
            Mode::Irq => 0b10010,
            Mode::Svc => 0b10011,
            Mode::Abt => 0b10111,
            Mode::Und => 0b11011,
            Mode::Sys => 0b11111,
        };
        self.regs[REG_CPSR] = (self.regs[REG_CPSR] & !0b11111) | bits;
    }

    #[inline(always)]
    fn get_state(&self) -> State {
        match (self.regs[REG_CPSR] & 0b100000) == 0 {
            true => State::Arm,
            false => State::Thumb,
        }
    }
}
