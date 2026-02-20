use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{Error, Read, Write};
use std::path::Path;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
}

impl From<Register> for u8 {
    fn from(val: Register) -> u8 {
        val as u8
    }
}

impl From<Register> for usize {
    fn from(val: Register) -> usize {
        val as usize
    }
}

fn convert_str_to_i16_vec(str: &str) -> Vec<i16> {
    let mut res = Vec::with_capacity(str.len());
    for s in str.bytes() {
        res.push(s as i16);
    }

    res
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Instruction(pub i16);

// this could be implemented as a enum, but whatever.
// A lot of repeated code here, cleanup one day
// DR and SR are always 3 bits.
impl Instruction {
    // all inputs should use at most 3 bits.
    pub fn add(dr: impl Into<u8>, sr1: impl Into<u8>, sr2: impl Into<u8>) -> Self {
        let dr = dr.into();
        let sr1 = sr1.into();
        let sr2 = sr2.into();

        assert!(dr < 8 && sr1 < 8 && sr2 < 8);
        // 15-12  11-9  8-6         2-0
        // 0001    DR   SR1  0  00  SR2

        let mut instr: i16 = 1 << 12;

        instr |= (dr as i16) << 9;
        instr |= (sr1 as i16) << 6;
        instr |= sr2 as i16;

        Instruction(instr)
    }

    // imm must only use up to 5 bits
    pub fn add_imm(dr: impl Into<u8>, sr1: impl Into<u8>, imm: i8) -> Self {
        let dr = dr.into();
        let sr1 = sr1.into();

        assert!(dr < 8 && sr1 < 8 && (-8..=7).contains(&imm));
        // 15-12  11-9  8-6     4-0
        // 0001    DR   SR1  1  Imm5

        let mut instr: i16 = 1 << 12;

        instr |= (dr as i16) << 9;
        instr |= (sr1 as i16) << 6;

        instr |= 1 << 5;

        instr |= Self::convert_immediate_byte_to_nybble(imm) as i16;

        Instruction(instr)
    }

    pub fn is_add(&self) -> bool {
        self.check_header(0b0001)
    }

    // all inputs should use at most 3 bits.
    // 15-12  11-9  8-6         2-0
    // 0101    DR   SR1  0  00  SR2
    pub fn and(dr: impl Into<u8>, sr1: impl Into<u8>, sr2: impl Into<u8>) -> Self {
        let dr = dr.into();
        let sr1 = sr1.into();
        let sr2 = sr2.into();

        assert!(dr < 8 && sr1 < 8 && sr2 < 8);

        let mut instr: i16 = (1 << 14) | (1 << 12);

        instr |= (dr as i16) << 9;
        instr |= (sr1 as i16) << 6;
        instr |= sr2 as i16;

        Instruction(instr)
    }

    // imm must only use up to 5 bits
    // 15-12  11-9  8-6     4-0
    // 0101    DR   SR1  1  Imm5
    pub fn and_imm(dr: impl Into<u8>, sr1: impl Into<u8>, imm: i8) -> Self {
        let dr = dr.into();
        let sr1 = sr1.into();

        assert!(dr < 8 && sr1 < 8 && (-8..=7).contains(&imm));

        let mut instr: i16 = 0b0101 << 12;

        instr |= (dr as i16) << 9;
        instr |= (sr1 as i16) << 6;

        instr |= 1 << 5;

        instr |= Self::convert_immediate_byte_to_nybble(imm) as i16;

        Instruction(instr)
    }

    pub fn is_and(&self) -> bool {
        self.check_header(0b0101)
    }

    // BR 	0000 	n 	z	p 	PCoffset9
    // flags should use only 3 bits, each representing a condition code
    pub fn branch(flags: u8, ip_offset: i8) -> Self {
        assert!(flags < 8);

        let mut instr: i16 = ((flags & 0b111) as i16) << 9;

        instr |= (ip_offset as i16) & 0b11111111;

        Instruction(instr)
    }

    // returns flags and the ip offset separately.
    pub fn get_branch(&self) -> Option<(u8, i8)> {
        if self.check_header(0b0000) {
            Some((
                (((self.0 as u16) >> 9) & 0b111) as u8,
                (self.0 & 0b11111111) as i8,
            ))
        } else {
            None
        }
    }

    // JMP 	1100 	000  BaseR 	000000
    pub fn jmp(reg: impl Into<u8>) -> Self {
        let mut instr: i16 = 0b1100 << 12;

        instr |= (reg.into() as i16) << 6;

        Instruction(instr)
    }

    pub fn get_jmp(&self) -> Option<Register> {
        if self.check_header(0b1100) {
            Some((((self.0 >> 6) & 0b111) as u8).into())
        } else {
            None
        }
    }

    // TODO
    // JSR 	0100 	1 	 PCoffset11
    // JSRR	0100 	0 	 00 	BaseR 	000000

    // LD  	0010 	DR 	 PCoffset9
    pub fn ld(dr: impl Into<u8>, ip_offset: i8) -> Self {
        let dr = dr.into();

        assert!(dr < 8);

        let mut instr: i16 = 0b0010 << 12;

        instr |= (dr as i16) << 9;
        instr |= (ip_offset as i16) & 0b111111111; // casting from i8 to i16 adds leading 1s

        Instruction(instr)
    }

    pub fn get_ld(&self) -> Option<(u8, i8)> {
        if self.check_header(0b0010) {
            Some((((self.0 >> 9) & 0b111) as u8, (self.0 & 0b11111111) as i8))
        } else {
            None
        }
    }

    // TODO
    // LDI 	1010 	DR 	 PCoffset9
    // LDR 	0110 	DR 	 BaseR 	offset6

    // LEA 	1110 	DR 	 PCoffset9
    pub fn lea(dr: impl Into<u8>, ip_offset: i8) -> Self {
        let dr = dr.into();
        assert!(dr < 8);

        let mut instr: i16 = 0b1110 << 12;

        instr |= (dr as i16) << 9;
        instr |= (ip_offset as i16) & 0b11111111;

        Instruction(instr)
    }

    pub fn get_lea(&self) -> Option<(Register, i8)> {
        if self.check_header(0b1110) {
            Some(
                (
                    (((self.0 >> 9) & 0b111) as u8).into(),
                    (self.0 & 0b11111111) as i8,
                )
            )
        } else {
            None
        }
    }

    // NOT 	1001 	DR 	 SR 	111111
    pub fn not(dr: impl Into<u8>, sr: impl Into<u8>) -> Self {
        let dr = dr.into();
        let sr = sr.into();

        assert!(dr < 8 && sr < 8);

        let mut instr: i16 = 0b1001 << 12;

        instr |= (dr as i16) << 9;
        instr |= (sr as i16) << 6;
        instr |= 0b111111;

        Instruction(instr)
    }

    pub fn is_not(&self) -> bool {
        self.check_header(0b1001)
    }

    // RET 	1100 	000  111 	000000
    // ST 	0011 	SR 	 PCoffset9
    // STI 	1011 	SR 	 PCoffset9
    // STR 	0111 	SR 	 BaseR 	offset6

    // TRAP	1111 	0000 trapvect8
    pub fn trap(vector: u8) -> Self {
        let mut instr: i16 = 0b1111 << 12;

        instr |= vector as i16;

        Instruction(instr)
    }

    pub fn get_trap_vector(&self) -> Option<u8> {
        if ((self.0 as u16) >> 8) == 0b11110000 {
            Some((self.0 & 0b11111111) as u8)
        } else {
            None
        }
    }

    // source for the following trap vectors: https://acg.cis.upenn.edu/milom/cse240-Fall05/handouts/Ch09-a.pdf
    pub fn trap_get_c() -> Self {
        Self::trap(0x20)
    }

    pub fn trap_out() -> Self {
        Self::trap(0x21)
    }

    pub fn trap_puts() -> Self {
        Self::trap(0x22)
    }

    pub fn trap_in() -> Self {
        Self::trap(0x23)
    }

    pub fn trap_halt() -> Self {
        // yes 0x24 is skipped
        Self::trap(0x25)
    }

    // reserved 1101

    // a nybble is a 4-bit number.
    fn convert_immediate_byte_to_nybble(imm: i8) -> u8 {
        let mut value = (imm as u8) & 0b00000111; // mask first 3 bits

        if imm < 0 {
            value |= 0b00001000; // set 4th bit to 1
        }

        value
    }

    fn get_dr_sr1_sr2(&self) -> (u8, u8, u8) {
        // & to mask out the rest of the bits
        // no need to convert to u16 here since we mask out the extra bits anyway
        (
            ((self.0 >> 9) & 0b111) as u8,
            ((self.0 >> 6) & 0b111) as u8,
            (self.0 & 0b111) as u8,
        )
    }

    fn get_dr_sr1_imm5(&self) -> (u8, u8, u8) {
        (
            ((self.0 >> 9) & 0b111) as u8,
            ((self.0 >> 6) & 0b111) as u8,
            (self.0 & 0b11111) as u8,
        )
    }

    fn get_dr_sr(&self) -> (u8, u8) {
        (((self.0 >> 9) & 0b111) as u8, ((self.0 >> 6) & 0b111) as u8)
    }

    fn check_bit_5(&self) -> bool {
        ((self.0 >> 5) & 0b1) != 0
    }

    fn check_header(&self, header: u16) -> bool {
        // convert to unsigned, since shifting to the right with a negative number adds leading 1s.
        ((self.0 as u16) >> 12) == header
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "0x{:x}", self.0)
        } else {
            write!(f, "0b{:016b}", self.0)
        }
    }
}

#[derive(Default, Debug)]
struct Registers {
    reg: [i16; 8],
}

impl From<u8> for Register {
    fn from(value: u8) -> Self {
        match value {
            0 => Register::R0,
            1 => Register::R1,
            2 => Register::R2,
            3 => Register::R3,
            4 => Register::R4,
            5 => Register::R5,
            6 => Register::R6,
            7 => Register::R7,

            _ => panic!("Invalid register: {}", value), // todo print machine state
        }
    }
}

impl Registers {
    fn get(&self, i: Register) -> i16 {
        let i: usize = i.into();
        self.reg[i]
    }

    fn get_mut(&mut self, i: Register) -> &mut i16 {
        let i: usize = i.into();
        &mut self.reg[i]
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ConditionCode {
    // hmm i thought there was a carry flag
    Negative,
    Zero,
    Positive,
}

impl ConditionCode {
    fn into_flags(self) -> u8 {
        match self {
            ConditionCode::Negative => 0b100,
            ConditionCode::Zero => 0b010,
            ConditionCode::Positive => 0b001,
        }
    }
}

pub struct Machine<'a> {
    registers: Registers,
    memory: Vec<i16>,
    ip: u16, // LC-3 is word addressable.
    condition_code: ConditionCode,

    halted: bool,
    jumped: bool,

    _stdin: Box<dyn Read + 'a>,
    stdout: Box<dyn Write + 'a>,
}

// Not sure if the condition code should start as the Zero flag.
// according to https://www.cs.utexas.edu/~fussell/courses/cs310h/lectures/Lecture_10-310h.pdf it states
// that exactly one condition code is set at all times. I suppose Zero is a sensible default.
impl<'a> Machine<'a> {
    pub fn new_std(instructions: &[Instruction]) -> Self {
        Self::new(std::io::stdin(), std::io::stdout(), 0x3000, instructions)
    }

    pub fn new(read: impl Read + 'a, write: impl Write + 'a, orig: u16, instructions: &[Instruction]) -> Self {
        let mut memory = Vec::from_iter((0..orig).map(|_| 0)); // instructions start at 0x3000.
        for inst in instructions {
            memory.push(inst.0);
        }

        Self {
            registers: Registers::default(),
            memory,
            ip: orig,
            condition_code: ConditionCode::Zero,
            halted: false,
            jumped: false,
            _stdin: Box::new(read),
            stdout: Box::new(write),
        }
    }

    pub fn set_memory_at(&mut self, index: u16, value: i16) {
        self.memory[index as usize] = value;
    }

    // maybe check if length is greater than the max of an i16?
    pub fn ensure_memory_space_at(&mut self, index: usize) {
        if index >= self.memory.len() {
            self.memory.resize(index + 1, 0);
        }
    }

    pub fn set_span_at(&mut self, index: u16, value: &[i16]) {
        let mut value_index = 0;
        for i in (index as usize)..(index as usize + value.len()) {
            self.ensure_memory_space_at(i);

            self.memory[i] = value[value_index];
            value_index += 1;
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
        let instr = self.memory[self.ip as usize];
        self.ip += 1; // ip points to the next instruction
        self.evaluate(Instruction(instr));
    }

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
        }
        //...
        else if let Some((dr, offset)) = instr.get_ld() {
            // cast to i32 so that subtraction can be done properly
            let value = self.memory[((self.ip as i32) + (offset as i32)) as usize];
            *self.registers.get_mut(dr.into()) = value;

            self.set_condition_code_based_on(dr.into());
        }
        // ...
        else if let Some((dr, offset)) = instr.get_lea() {
            let effective_addr = ((self.ip as i32) + (offset as i32)) as i16;
            *self.registers.get_mut(dr.into()) = effective_addr;
        }
        // ...
        else if instr.is_not() {
            self.handle_not(instr);
        }
        // ...
        else if let Some(vec) = instr.get_trap_vector() {
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
            0x20 => todo!(),
            0x21 => {
                let r0 = self.registers.get(Register::R0);
                self.stdout
                    .write_all(&[r0 as u8])
                    .expect("Failed to write to stdout");
            }
            0x22 => {
                let mut addr = self.registers.get(Register::R0) as usize;
                self.ensure_memory_space_at(addr);

                while self.memory[addr] != 0 {
                    self.stdout
                        .write_all(&[self.memory[addr] as u8])
                        .expect("Failed to write to stdout");

                    addr += 1;
                    self.ensure_memory_space_at(addr);
                }
                self.stdout.flush().expect("Failed to flush stdout");
            },
            0x23 => todo!(),
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

fn read_binary_file(file: &Path) -> Vec<Instruction> {
    let mut res = Vec::new();

    let mut file = File::open(file).unwrap();
    loop {
        let mut buf = [0u8; 2];
        if let Err(err) = file.read_exact(&mut buf) {
            break;
        }
        let instr = Instruction(((buf[0] as i16) << 8) | ((buf[1] as i16) & 0b11111111));
        res.push(instr);
    }

    res
}

fn main() {
    let binary = read_binary_file(&Path::new("hello.obj"));
    let orig = binary[0].0;

    let mut machine = Machine::new(
        std::io::stdin(),
        std::io::stdout(),
        orig as u16,
        &binary[1..]
    );

    machine.run_until_halt();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufWriter;

    #[test]
    fn add_instr() {
        let add = Instruction::add(Register::R0, Register::R1, Register::R2);
        let add_imm = Instruction::add_imm(Register::R0, Register::R1, 5);

        assert_eq!(format!("{add}"), "0b0001000001000010");
        assert_eq!(format!("{add_imm}"), "0b0001000001100101");

        assert!(add.is_add());
        assert!(add_imm.is_add());
    }

    #[test]
    fn and_instr() {
        let and = Instruction::and(Register::R0, Register::R1, Register::R2);
        let and_imm = Instruction::and_imm(Register::R0, Register::R1, 5);

        assert_eq!(format!("{and}"), "0b0101000001000010");
        assert_eq!(format!("{and_imm}"), "0b0101000001100101");

        assert!(and.is_and());
        assert!(and_imm.is_and());
    }

    #[test]
    fn not_instr() {
        let not = Instruction::not(Register::R0, Register::R1);

        assert_eq!(format!("{not}"), "0b1001000001111111");
    }

    #[test]
    fn add_add() {
        let mut machine = Machine::new_std(&[
            Instruction::add_imm(Register::R0, Register::R1, 5), // r0 = 5
            Instruction::add_imm(Register::R1, Register::R0, 5), // r1 = 10
        ]);

        machine.step();
        machine.step();

        assert_eq!(machine.registers.get(Register::R0), 5);
        assert_eq!(machine.registers.get(Register::R1), 10);
    }

    #[test]
    fn add_add_and() {
        let mut machine = Machine::new_std(&[
            Instruction::add_imm(Register::R0, Register::R1, 5), // r0 = 5
            Instruction::add_imm(Register::R1, Register::R1, 5), // r1 = 5
            Instruction::and(Register::R2, Register::R0, Register::R1), // r2 = 5 (r0 & r1)
        ]);

        machine.step();
        machine.step();
        machine.step();

        assert_eq!(machine.registers.get(Register::R0), 5);
        assert_eq!(machine.registers.get(Register::R1), 5);
        assert_eq!(machine.registers.get(Register::R2), 5);
    }

    #[test]
    fn add_not() {
        let mut machine = Machine::new_std(&[
            Instruction::add_imm(Register::R0, Register::R1, 5), // r0 = 5
            Instruction::not(Register::R1, Register::R0),        // r2 = 1111111111111010 = -6 (!r0)
        ]);

        machine.step();
        machine.step();

        assert_eq!(machine.registers.get(Register::R0), 5);
        assert_eq!(machine.registers.get(Register::R1), -6);
    }

    #[test]
    fn print_a() {
        let mut output = BufWriter::new(Vec::new());

        let mut machine = Machine::new(
            std::io::stdin(),
            &mut output,
            0x3000,
            &[
                // largest immediate we can do is 7
                // yes this can be condensed
                Instruction::add_imm(Register::R0, Register::R1, 7), // r0 = 7
                Instruction::add_imm(Register::R1, Register::R1, 7), // r1 = 7
                Instruction::add(Register::R0, Register::R0, Register::R1), // r0 = r0 + r1 (14)
                Instruction::add(Register::R0, Register::R0, Register::R1), // r0 = r0 + r1 (21)
                Instruction::add(Register::R0, Register::R0, Register::R1), // r0 = r0 + r1 (28)
                Instruction::add(Register::R0, Register::R0, Register::R1), // r0 = r0 + r1 (35)
                Instruction::add(Register::R0, Register::R0, Register::R1), // r0 = r0 + r1 (42)
                Instruction::add(Register::R0, Register::R0, Register::R1), // r0 = r0 + r1 (49)
                Instruction::add(Register::R0, Register::R0, Register::R1), // r0 = r0 + r1 (56)
                Instruction::add(Register::R0, Register::R0, Register::R1), // r0 = r0 + r1 (63)
                Instruction::add_imm(Register::R0, Register::R0, 2), // r0 = r0 + 2 (65, 'A' in ASCII)
                Instruction::trap_out(),                             // print r0
                Instruction::trap_halt(),
            ],
        );

        machine.run_until_halt();
        drop(machine);

        let buf = output.into_inner().unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert_eq!(output, "A");
    }

    #[test]
    fn check_branching() {
        let mut machine = Machine::new_std(&[
            Instruction::add_imm(Register::R0, Register::R1, 7), // r0 = 7 // flag = p
            Instruction::branch(0b001, 1), // check if positive, then skip over the next instruction
            Instruction::trap_halt(),
            Instruction::add_imm(Register::R0, Register::R0, 7), // r0 = 14
            Instruction::trap_halt(),
        ]);
        machine.run_until_halt();

        assert_eq!(machine.registers.get(Register::R0), 14);

        let mut machine = Machine::new_std(&[
            Instruction::add_imm(Register::R0, Register::R1, 7), // r0 = 7 // flag = p
            Instruction::branch(0b110, 1), // check if negative or zero (false), so we don't jump
            Instruction::trap_halt(),
            Instruction::add_imm(Register::R0, Register::R0, 7), // r0 = 14
            Instruction::trap_halt(),
        ]);
        machine.run_until_halt();

        assert_eq!(machine.registers.get(Register::R0), 7);

        let mut machine = Machine::new_std(&[
            Instruction::branch(0b111, -1), // check if negative or zero (false), so we don't jump
        ]);

        machine.step();

        assert_eq!(machine.ip, 0x3000);
    }

    #[test]
    fn check_jmp() {
        let mut machine = Machine::new_std(&[
            Instruction::add_imm(Register::R0, Register::R1, 4), // r0 = 4
            Instruction::jmp(Register::R0),                      // jmp to 4 (r0 = 4)
            Instruction::trap_halt(), // this should not happen since we jumped over it
        ]);

        machine.step();
        machine.step();

        assert_eq!(machine.ip, 4);
    }

    #[test]
    fn check_ld() {
        let mut machine = Machine::new_std(&[
            Instruction::ld(Register::R0, -2), // r0 = 50 (see code after this)
            Instruction::trap_halt(),
        ]);

        machine.set_memory_at(0x3000 - 1, 50);
        machine.run_until_halt();

        assert_eq!(machine.registers.get(Register::R0), 50);
    }

    #[test]
    fn hello_world() { // TODO
        let mut output = BufWriter::new(Vec::new());

        let mut machine = Machine::new(
            std::io::stdin(),
            &mut output,
            0x3000,
            &[
                Instruction::lea(Register::R0, 2), // r0 = text_addr
                Instruction::trap_puts(), // print string stored at address in r0
                Instruction::trap_halt(),
            ],
        );

        let text = "Hello, world!\n";
        let text_addr = 0x3003;
        machine.string_set(text_addr, text);

        machine.run_until_halt();

        drop(machine);

        assert_eq!(String::from_utf8(output.into_inner().unwrap()).unwrap(), text);
    }
}
