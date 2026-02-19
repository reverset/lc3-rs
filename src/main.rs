use std::fmt::{Display, Formatter};

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

impl Into<u8> for Register {
    fn into(self) -> u8 {
        self as u8
    }
}

impl Into<usize> for Register {
    fn into(self) -> usize {
        self as usize
    }
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
    pub fn add_imm(dr: impl Into<u8>, sr1: impl Into<u8>, imm: u8) -> Self {
        let dr = dr.into();
        let sr1 = sr1.into();

        assert!(dr < 8 && sr1 < 8 && imm < 32);
        // 15-12  11-9  8-6     4-0
        // 0001    DR   SR1  1  Imm5

        let mut instr: i16 = 1 << 12;

        instr |= (dr as i16) << 9;
        instr |= (sr1 as i16) << 6;

        instr |= 1 << 5;

        instr |= imm as i16;

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
    pub fn and_imm(dr: impl Into<u8>, sr1: impl Into<u8>, imm: u8) -> Self {
        let dr = dr.into();
        let sr1 = sr1.into();

        assert!(dr < 8 && sr1 < 8 && imm < 32);

        let mut instr: i16 = 0b0101 << 12;

        instr |= (dr as i16) << 9;
        instr |= (sr1 as i16) << 6;

        instr |= 1 << 5;

        instr |= imm as i16;

        Instruction(instr)
    }

    pub fn is_and(&self) -> bool {
        self.check_header(0b0101)
    }

    // TODO
    // BR 	0000 	n 	z	p 	PCoffset9
    // JMP 	1100 	000  BaseR 	000000
    // JSR 	0100 	1 	 PCoffset11
    // JSRR	0100 	0 	 00 	BaseR 	000000
    // LD  	0010 	DR 	 PCoffset9
    // LDI 	1010 	DR 	 PCoffset9
    // LDR 	0110 	DR 	 BaseR 	offset6
    // LEA 	1110 	DR 	 PCoffset9
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
    // reserved 1101

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
        (
            ((self.0 >> 9) & 0b111) as u8,
            ((self.0 >> 6) & 0b111) as u8,
        )
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
enum ConditionCode { // hmm i thought there was a carry flag
    Negative,
    Zero,
    Positive,
}

pub struct Machine {
    registers: Registers,
    memory: Vec<i16>,
    ip: u16, // LC-3 is word addressable.
    condition_code: ConditionCode,
}

// Not sure if the condition code should start as the Zero flag.
// according to https://www.cs.utexas.edu/~fussell/courses/cs310h/lectures/Lecture_10-310h.pdf it states
// that exactly one condition code is set at all times. I suppose Zero is a sensible default.
impl Machine {
    pub fn new(instructions: &[Instruction]) -> Self {
        let mut memory = Vec::from_iter((0..0x3000).map(|_| 0)); // instructions start at 0x3000.
        for inst in instructions {
            memory.push(inst.0);
        }

        Self {
            registers: Registers::default(),
            memory,
            ip: 0x3000,
            condition_code: ConditionCode::Zero,
        }
    }

    pub fn step(&mut self) {
        self.evaluate_at_ip();

        self.ip += 1;
    }

    pub fn evaluate_at_ip(&mut self) {
        let instr = self.memory[self.ip as usize];
        self.evaluate(Instruction(instr));
    }

    pub fn evaluate(&mut self, instr: Instruction) {
        if instr.is_add() {
            self.handle_add(instr);
        } else if instr.is_and() {
            self.handle_and(instr);
        }
        // ...
        else if instr.is_not() {
            self.handle_not(instr);
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

    fn set_condition_code_based_on(&mut self, reg: Register) {
        self.condition_code = match self.registers.get(reg) {
            0 => ConditionCode::Zero,
            0.. => ConditionCode::Positive,
            ..0 => ConditionCode::Negative,
        }
    }
}

fn main() {
    let mut machine = Machine::new(&[
        Instruction::add_imm(Register::R0, Register::R1, 5), // r0 = 5
        Instruction::add_imm(Register::R1, Register::R0, 5), // r1 = 10
    ]);

    machine.step();
    machine.step();

    println!("{:?}", machine.registers);
}


#[cfg(test)]
mod tests {
    use super::*;

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
        let mut machine = Machine::new(&[
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
        let mut machine = Machine::new(&[
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
        let mut machine = Machine::new(&[
            Instruction::add_imm(Register::R0, Register::R1, 5), // r0 = 5
            Instruction::not(Register::R1, Register::R0), // r2 = 1111111111111010 = -6 (!r0)
        ]);

        machine.step();
        machine.step();

        assert_eq!(machine.registers.get(Register::R0), 5);
        assert_eq!(machine.registers.get(Register::R1), -6);
    }
}

