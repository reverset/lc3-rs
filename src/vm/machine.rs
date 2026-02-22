use crate::bit_util::convert_str_to_i16_vec;
use crate::vm::instructions::Instruction::{
    Add, AddImmediate, And, AndImmediate, Branch, Jump, JumpSubroutine, JumpSubroutineRegister,
    Load, LoadEffectiveAddress, LoadIndirect, LoadRegister, Not, Reserved, ReturnToInterrupt,
    Store, StoreIndirect, StoreRegister, Trap,
};
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
        let mut memory = Vec::from_iter((0..orig).map(|_| 0));
        for inst in instructions {
            memory.push(inst.encode() as i16);
        }

        Self {
            registers: Registers::default(),
            memory: Memory(memory),
            ip: orig,
            condition_code: ConditionCode::Zero,
            halted: false,
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
        self.evaluate(Instruction::decode(instr as u16));
    }

    pub fn add_to_ip(&mut self, offset: i16) {
        self.ip.wrapping_add_signed(offset);
    }

    // cleanup needed
    pub fn evaluate(&mut self, instr: Instruction) {
        match instr {
            Add(dest, s1, s2) => {
                let s1 = self.registers.get(s1);
                let s2 = self.registers.get(s2);

                *self.registers.get_mut(dest) = s1.wrapping_add(s2);
                self.set_condition_code_based_on(dest);
            }

            AddImmediate(dest, s1, imm5) => {
                let s1 = self.registers.get(s1);

                *self.registers.get_mut(dest) = s1.wrapping_add(imm5.into_inner() as i16);
                self.set_condition_code_based_on(dest);
            }

            And(dest, s1, s2) => {
                let s1 = self.registers.get(s1);
                let s2 = self.registers.get(s2);

                *self.registers.get_mut(dest) = s1 & s2;
                self.set_condition_code_based_on(dest);
            }

            AndImmediate(dest, s1, imm5) => {
                let s1 = self.registers.get(s1);

                *self.registers.get_mut(dest) = s1 & (imm5.into_inner() as i16);
                self.set_condition_code_based_on(dest);
            }

            Branch(flags, offset) => {
                if self.condition_code.into_flags() & flags.into_flags() != 0 {
                    self.ip = self.ip.wrapping_add_signed(offset.into_inner());
                }
            }

            Jump(register) => {
                self.ip = self.registers.get(register) as u16;
            }

            JumpSubroutine(offset) => {
                *self.registers.get_mut(Register::R7) = self.ip as i16;
                self.ip = self.ip.wrapping_add_signed(offset.into_inner());
            }

            JumpSubroutineRegister(baser) => {
                *self.registers.get_mut(Register::R7) = self.ip as i16;
                self.ip = self.registers.get(baser) as u16;
            }

            Load(dest, offset) => {
                let addr = self.ip.wrapping_add_signed(offset.into_inner());
                let val = self.memory[addr];

                *self.registers.get_mut(dest) = val;
                self.set_condition_code_based_on(dest);
            }

            LoadIndirect(dest, offset) => {
                let addr = self.ip.wrapping_add_signed(offset.into_inner());
                let addr = self.memory[addr];

                let val = self.memory[addr as u16];
                *self.registers.get_mut(dest) = val;
                self.set_condition_code_based_on(dest);
            }

            LoadRegister(dest, baser, offset) => {
                let addr = (self.registers.get(baser) as u16)
                    .wrapping_add_signed(offset.into_inner() as i16);
                let val = self.memory[addr];

                *self.registers.get_mut(dest) = val;
                self.set_condition_code_based_on(dest);
            }

            LoadEffectiveAddress(dest, offset) => {
                let ea = self.ip.wrapping_add_signed(offset.into_inner());

                *self.registers.get_mut(dest) = ea as i16;
                self.set_condition_code_based_on(dest);
            }

            Not(dest, source) => {
                let source = self.registers.get(source);

                *self.registers.get_mut(dest) = !source;
                self.set_condition_code_based_on(dest);
            }

            // RET is just JMP
            ReturnToInterrupt => {
                todo!("RTI not implemented")
            }

            Store(source, offset) => {
                let addr = self.ip.wrapping_add_signed(offset.into_inner());
                self.memory[addr] = self.registers.get(source);
            }

            StoreIndirect(source, offset) => {
                let addr = self.ip.wrapping_add_signed(offset.into_inner());
                let addr = self.memory[addr] as u16;

                self.memory[addr] = self.registers.get(source);
            }

            StoreRegister(source, baser, offset) => {
                let addr = self.registers.get(baser) as u16;
                let addr = addr.wrapping_add_signed(offset.into_inner() as i16);

                self.memory[addr] = self.registers.get(source);
            }

            Trap(vector) => self.handle_trap(vector),

            Reserved => todo!("reserved not implemented"),
        }
    }

    fn handle_trap(&mut self, vec: u8) {
        match vec {
            // getc FIXME: this should not care for the new line at the end. It's more of a 'wait until pressed' kind of thing
            0x20 => {
                self.stdout.flush().unwrap();
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
                    .write_all(&[(r0 & 0b11111111) as u8])
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
