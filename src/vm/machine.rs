use crate::bit_util::convert_str_to_i16_vec;
use crate::vm::instructions::{Instruction, Register, Registers};
use std::io::{Read, Write};
use std::ops::{Index, IndexMut};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConditionCode {
    // hmm i thought there was a carry flag
    Negative,
    Zero,
    Positive,
}

impl ConditionCode {
    pub fn into_flags(self) -> u8 {
        match self {
            ConditionCode::Negative => 0b100,
            ConditionCode::Zero => 0b010,
            ConditionCode::Positive => 0b001,
        }
    }
}

pub struct Memory(Vec<i16>);

impl Memory {
    pub fn resize(&mut self, size: usize, val: i16) {
        self.0.resize(size, val);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn ensure_space(&mut self, index: u16) {
        if index as usize >= self.len() {
            self.resize(index as usize + 1, 0);
        }
    }
}

impl Index<u16> for Memory {
    type Output = i16;

    fn index(&self, index: u16) -> &Self::Output {
        if index as usize >= self.len() {
            &0
        } else {
            self.0.index(index as usize)
        }
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        self.ensure_space(index);
        &mut self.0[index as usize]
    }
}

pub struct Machine<'a> {
    pub registers: Registers,
    pub memory: Memory,
    pub ip: u16, // LC-3 is word addressable.
    pub condition_code: ConditionCode,

    pub halted: bool,
    pub jumped: bool,

    pub stdin: Box<dyn Read + 'a>,
    pub stdout: Box<dyn Write + 'a>,
}

// Not sure if the condition code should start as the Zero flag.
// according to https://www.cs.utexas.edu/~fussell/courses/cs310h/lectures/Lecture_10-310h.pdf it states
// that exactly one condition code is set at all times. I suppose Zero is a sensible default.
#[allow(unused)]
impl<'a> Machine<'a> {
    pub fn new_std(instructions: &[Instruction]) -> Self {
        Self::new(std::io::stdin(), std::io::stdout(), 0x3000, instructions)
    }

    pub fn new(
        read: impl Read + 'a,
        write: impl Write + 'a,
        orig: u16,
        instructions: &[Instruction],
    ) -> Self {
        let mut memory = Vec::from_iter((0..orig).map(|_| 0)); // instructions start at 0x3000.
        for inst in instructions {
            memory.push(inst.0);
        }

        Self {
            registers: Registers::default(),
            memory: Memory(memory),
            ip: orig,
            condition_code: ConditionCode::Zero,
            halted: false,
            jumped: false,
            stdin: Box::new(read),
            stdout: Box::new(write),
        }
    }

    pub fn set_memory_at(&mut self, index: u16, value: i16) {
        self.memory[index] = value;
    }

    pub fn set_span_at(&mut self, index: u16, value: &[i16]) {
        for (value_index, i) in (index..(index + value.len() as u16)).enumerate() {
            self.memory[i] = value[value_index];
        }
    }

    pub fn string_set(&mut self, index: u16, value: &str) {
        self.set_span_at(index, &convert_str_to_i16_vec(value));
    }

    pub fn run_until_halt(&mut self) {
        while !self.halted {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let instr = self.memory[self.ip];
        self.ip += 1; // ip points to the next instruction
        self.evaluate(Instruction(instr));
    }

    // cleanup needed
    pub fn evaluate(&mut self, instr: Instruction) {
        if instr.is_add() {
            self.handle_add(instr);
        } else if instr.is_and() {
            self.handle_and(instr);
        } else if let Some((flags, offset)) = instr.get_branch() {
            // at least one '1' matches with the condition flags
            if (self.condition_code.into_flags() & flags) != 0 {
                self.ip = (self.ip as i32 + offset as i32) as u16;
                self.jumped = true;
            }
        } else if let Some(reg) = instr.get_jmp() {
            self.ip = self.registers.get(reg) as u16;
            self.jumped = true;
        } else if let Some(offset) = instr.get_jsr() {
            *self.registers.get_mut(Register::R7) = self.ip as i16;
            self.ip = ((self.ip as i32) + (offset as i32)) as u16;
        } else if let Some(baser) = instr.get_jsrr() {
            *self.registers.get_mut(Register::R7) = self.ip as i16;

            let addr = self.registers.get(baser.into());
            self.ip = addr as u16;
        } else if let Some((dr, offset)) = instr.get_ld() {
            // cast to i32 so that subtraction can be done properly
            let value = self.memory[((self.ip as i32) + (offset as i32)) as u16];
            *self.registers.get_mut(dr.into()) = value;

            self.set_condition_code_based_on(dr.into());
        } else if let Some((dr, offset)) = instr.get_ldi() {
            let addr = self.memory[((self.ip as i32) + (offset as i32)) as u16];
            let value = self.memory[addr as u16];
            *self.registers.get_mut(dr.into()) = value;

            self.set_condition_code_based_on(dr.into());
        } else if let Some((dr, baser, offset)) = instr.get_ldr() {
            let addr = self.registers.get(baser.into()) + offset as i16;
            let value = self.memory[addr as u16];
            *self.registers.get_mut(dr.into()) = value;

            self.set_condition_code_based_on(dr.into());
        } else if let Some((dr, offset)) = instr.get_lea() {
            let effective_addr = ((self.ip as i32) + (offset as i32)) as i16;
            *self.registers.get_mut(dr) = effective_addr;

            self.set_condition_code_based_on(dr);
        }
        // ...
        else if instr.is_not() {
            self.handle_not(instr);
        }
        // missing RTI
        else if let Some((sr, offset)) = instr.get_st() {
            let addr = ((self.ip as i32) + (offset as i32)) as u16;
            self.memory[addr] = self.registers.get(sr.into());
            self.set_condition_code_based_on(sr.into());
        } else if let Some((sr, offset)) = instr.get_sti() {
            let addr = ((self.ip as i32) + (offset as i32)) as u16;
            let addr = self.memory[addr];
            self.memory[addr as u16] = self.registers.get(sr.into());
            self.set_condition_code_based_on(sr.into());
        } else if let Some((sr, baser, offset)) = instr.get_str() {
            let addr = self.registers.get(baser.into()) + offset as i16;
            self.memory[addr as u16] = self.registers.get(sr.into());
            self.set_condition_code_based_on(sr.into());
        } else if let Some(vec) = instr.get_trap_vector() {
            self.handle_trap(vec);
        }
    }

    fn handle_add(&mut self, instr: Instruction) {
        // if immediate
        if instr.check_bit_5() {
            let (dr, sr1, imm) = instr.get_dr_sr1_imm5();

            let sr1 = self.registers.get(sr1.into());

            *self.registers.get_mut(dr.into()) = sr1 + (imm as i16);
            self.set_condition_code_based_on(dr.into());
        } else {
            let (dr, sr1, sr2) = instr.get_dr_sr1_sr2();
            let sr1 = self.registers.get(sr1.into());
            let sr2 = self.registers.get(sr2.into());

            *self.registers.get_mut(dr.into()) = sr1 + sr2;
            self.set_condition_code_based_on(dr.into());
        }
    }

    // FIXME duplicate code
    fn handle_and(&mut self, instr: Instruction) {
        // if immediate
        if instr.check_bit_5() {
            let (dr, sr1, imm) = instr.get_dr_sr1_imm5();

            let sr1 = self.registers.get(sr1.into());

            *self.registers.get_mut(dr.into()) = sr1 & (imm as i16); // & instead of +
            self.set_condition_code_based_on(dr.into());
        } else {
            let (dr, sr1, sr2) = instr.get_dr_sr1_sr2();
            let sr1 = self.registers.get(sr1.into());
            let sr2 = self.registers.get(sr2.into());

            *self.registers.get_mut(dr.into()) = sr1 & sr2;

            self.set_condition_code_based_on(dr.into());
        }
    }

    fn handle_not(&mut self, instr: Instruction) {
        let (dr, sr) = instr.get_dr_sr();
        let sr = self.registers.get(sr.into());

        *self.registers.get_mut(dr.into()) = !sr;
        self.set_condition_code_based_on(dr.into());
    }

    fn handle_trap(&mut self, vec: u8) {
        match vec {
            // getc
            0x20 => {
                let mut buf = [0u8; 1]; // only reads 1 ASCII char (7-bits)
                self.stdin
                    .read_exact(&mut buf)
                    .expect("failed to read stdin");

                *self.registers.get_mut(Register::R0) = buf[0] as i16;
            }
            // out
            0x21 => {
                let r0 = self.registers.get(Register::R0);
                self.stdout
                    .write_all(&[r0 as u8])
                    .expect("Failed to write to stdout");
            }
            // puts
            0x22 => {
                let mut addr = self.registers.get(Register::R0) as u16;

                while self.memory[addr] != 0 {
                    self.stdout
                        .write_all(&[self.memory[addr] as u8])
                        .expect("Failed to write to stdout");

                    addr += 1;
                }
                self.stdout.flush().expect("Failed to flush stdout");
            }
            // in
            0x23 => todo!(),
            // 0x24 putsp refer to ISA TODO

            // halt
            0x25 => {
                self.halted = true;
            }
            _ => todo!(),
        }
    }

    fn set_condition_code_based_on(&mut self, reg: Register) {
        self.condition_code = match self.registers.get(reg) {
            0 => ConditionCode::Zero,
            1.. => ConditionCode::Positive,
            ..0 => ConditionCode::Negative,
        }
    }
}
