use crate::bit_util::{
    check_i5_range, check_i6_range, check_i9_range, check_i11_range, i5_to_i8, i6_to_i8, i9_to_i16,
    i11_to_i16,
};
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Instruction(pub i16);

// this could be implemented as a enum, but whatever.
// A lot of repeated code here, cleanup one day
// DR and SR are always 3 bits.
#[allow(unused)]
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
    // 15-12  11-9  8-6     4-0
    // 0001    DR   SR1  1  Imm5
    pub fn add_imm(dr: impl Into<u8>, sr1: impl Into<u8>, imm: i8) -> Self {
        let dr = dr.into();
        let sr1 = sr1.into();

        assert!(dr < 8);
        assert!(sr1 < 8);
        check_i5_range(imm);

        let mut instr: i16 = 1 << 12;

        instr |= (dr as i16) << 9;
        instr |= (sr1 as i16) << 6;

        instr |= 1 << 5;

        instr |= (imm & 0b11111) as i16;

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

        assert!(dr < 8);
        assert!(sr1 < 8);
        check_i5_range(imm);

        let mut instr: i16 = 0b0101 << 12;

        instr |= (dr as i16) << 9;
        instr |= (sr1 as i16) << 6;

        instr |= 1 << 5;

        instr |= (imm & 0b11111) as i16;

        Instruction(instr)
    }

    pub fn is_and(&self) -> bool {
        self.check_header(0b0101)
    }

    // BR 	0000 	n 	z	p 	PCoffset9
    // flags should use only 3 bits, each representing a condition code
    pub fn branch(flags: u8, ip_offset: i16) -> Self {
        assert!(flags < 8);
        check_i9_range(ip_offset);

        let mut instr: i16 = ((flags & 0b111) as i16) << 9;

        instr |= ip_offset & 0b111111111;

        Instruction(instr)
    }

    // returns flags and the ip offset separately.
    pub fn get_branch(&self) -> Option<(u8, i16)> {
        if self.check_header(0b0000) {
            Some(((((self.0 as u16) >> 9) & 0b111) as u8, i9_to_i16(self.0)))
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

    // JSR 	0100 	1 	 PCoffset11
    pub fn jsr(offset: i16) -> Self {
        check_i11_range(offset);

        let mut instr: i16 = 0b0100 << 12;

        instr |= 1 << 11;
        instr |= offset & 0b11111111111;

        Instruction(instr)
    }

    pub fn get_jsr(&self) -> Option<i16> {
        if self.check_header(0b0100) && (self.0 & 0b100000000000 != 0) {
            Some(i11_to_i16(self.0))
        } else {
            None
        }
    }

    // JSRR	0100 	0 	 00 	BaseR 	000000
    pub fn jsrr(baser: impl Into<u8>) -> Self {
        let baser = baser.into();

        assert!(baser < 8);
        let mut instr: i16 = 0b0100 << 12;

        instr |= (baser as i16 & 0b111) << 6;
        Instruction(instr)
    }

    pub fn get_jsrr(&self) -> Option<u8> {
        if self.check_header(0b0100) && (self.0 & 0b100000000000 == 0) {
            Some(((self.0 >> 6) & 0b111) as u8)
        } else {
            None
        }
    }

    // LD  	0010 	DR 	 PCoffset9
    // OFFSET IS 9 BITS!!!!
    pub fn ld(dr: impl Into<u8>, ip_offset: i16) -> Self {
        let dr = dr.into();

        assert!(dr < 8);
        check_i9_range(ip_offset);

        let mut instr: i16 = 0b0010 << 12;

        instr |= (dr as i16) << 9;
        instr |= ip_offset & 0b111111111;

        Instruction(instr)
    }

    pub fn get_ld(&self) -> Option<(u8, i16)> {
        if self.check_header(0b0010) {
            Some((((self.0 >> 9) & 0b111) as u8, i9_to_i16(self.0)))
        } else {
            None
        }
    }

    // LDI 	1010 	DR 	 PCoffset9
    pub fn ldi(dr: impl Into<u8>, ip_offset: i16) -> Self {
        let dr = dr.into();

        assert!(dr < 8);
        check_i9_range(ip_offset);

        let mut instr: i16 = 0b1010 << 12;

        instr |= (dr as i16) << 9;
        instr |= ip_offset & 0b111111111;

        Instruction(instr)
    }

    pub fn get_ldi(&self) -> Option<(u8, i16)> {
        if self.check_header(0b1010) {
            Some(((((self.0) >> 9) & 0b111) as u8, i9_to_i16(self.0)))
        } else {
            None
        }
    }

    // LDR 	0110 	DR 	 BaseR 	offset6
    // offset is an i6
    pub fn ldr(dr: impl Into<u8>, baser: impl Into<u8>, offset: i8) -> Self {
        let dr = dr.into();
        let baser = baser.into();

        assert!(dr < 8);
        assert!(baser < 8);

        check_i6_range(offset);

        let mut instr: i16 = 0b0110 << 12;

        instr |= (dr as i16) << 9;
        instr |= (baser as i16) << 6;
        instr |= (offset as i16) & 0b111111;

        Instruction(instr)
    }

    pub fn get_ldr(&self) -> Option<(u8, u8, i8)> {
        if self.check_header(0b0110) {
            Some((
                ((self.0 >> 9) & 0b111) as u8,
                ((self.0 >> 6) & 0b111) as u8,
                i6_to_i8(self.0 as i8),
            ))
        } else {
            None
        }
    }

    // LEA 	1110 	DR 	 PCoffset9
    pub fn lea(dr: impl Into<u8>, ip_offset: i16) -> Self {
        let dr = dr.into();

        assert!(dr < 8);
        check_i9_range(ip_offset);

        let mut instr: i16 = 0b1110 << 12;

        instr |= (dr as i16) << 9;
        instr |= ip_offset & 0b111111111;

        Instruction(instr)
    }

    pub fn get_lea(&self) -> Option<(Register, i16)> {
        if self.check_header(0b1110) {
            Some(((((self.0 >> 9) & 0b111) as u8).into(), i9_to_i16(self.0)))
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
    pub fn ret() -> Self {
        Self::jmp(Register::R7)
    }

    // ST 	0011 	SR 	 PCoffset9
    pub fn st(sr: impl Into<u8>, offset: i16) -> Self {
        let sr = sr.into();

        assert!(sr < 8);
        check_i9_range(offset);

        let mut instr: i16 = 0b0011 << 12;
        instr |= (sr as i16) << 9;
        instr |= offset & 0b111111111;

        Instruction(instr)
    }

    pub fn get_st(&self) -> Option<(u8, i16)> {
        if self.check_header(0b0011) {
            Some((((self.0 >> 9) & 0b111) as u8, i9_to_i16(self.0)))
        } else {
            None
        }
    }

    // STI 	1011 	SR 	 PCoffset9
    pub fn sti(sr: impl Into<u8>, offset: i16) -> Self {
        let sr = sr.into();

        assert!(sr < 8);
        check_i9_range(offset);

        let mut instr: i16 = 0b1011 << 12;
        instr |= (sr as i16) << 9;
        instr |= offset & 0b111111111;

        Instruction(instr)
    }

    pub fn get_sti(&self) -> Option<(u8, i16)> {
        if self.check_header(0b1011) {
            Some((((self.0 >> 9) & 0b111) as u8, i9_to_i16(self.0)))
        } else {
            None
        }
    }

    // STR 	0111 	SR 	 BaseR 	offset6
    pub fn str(sr: impl Into<u8>, baser: impl Into<u8>, offset: i8) -> Self {
        let sr = sr.into();
        let baser = baser.into();

        assert!(sr < 8);
        assert!(baser < 8);
        check_i6_range(offset);

        let mut instr: i16 = 0b0111 << 12;
        instr |= (sr as i16) << 9;
        instr |= (baser as i16) << 6;
        instr |= (offset as i16) & 0b111111;

        Instruction(instr)
    }

    pub fn get_str(&self) -> Option<(u8, u8, i8)> {
        if self.check_header(0b0111) {
            Some((
                ((self.0 >> 9) & 0b111) as u8,
                ((self.0 >> 6) & 0b111) as u8,
                i6_to_i8(self.0 as i8),
            ))
        } else {
            None
        }
    }

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

    // TODO: refactor these to use the Option approach like the others
    pub fn get_dr_sr1_sr2(&self) -> (u8, u8, u8) {
        // & to mask out the rest of the bits
        // no need to convert to u16 here since we mask out the extra bits anyway
        (
            ((self.0 >> 9) & 0b111) as u8,
            ((self.0 >> 6) & 0b111) as u8,
            (self.0 & 0b111) as u8,
        )
    }

    pub fn get_dr_sr1_imm5(&self) -> (u8, u8, i8) {
        (
            ((self.0 >> 9) & 0b111) as u8,
            ((self.0 >> 6) & 0b111) as u8,
            i5_to_i8((self.0 as u16) as i8),
        )
    }

    pub fn get_dr_sr(&self) -> (u8, u8) {
        (((self.0 >> 9) & 0b111) as u8, ((self.0 >> 6) & 0b111) as u8)
    }

    pub fn check_bit_5(&self) -> bool {
        ((self.0 >> 5) & 0b1) != 0
    }

    pub fn check_header(&self, header: u16) -> bool {
        // convert to unsigned, since shifting to the right with a negative number adds leading 1s.
        (((self.0 as u16) >> 12) & 0b1111) == header
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
pub struct Registers {
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
    pub fn get(&self, i: Register) -> i16 {
        let i: usize = i.into();
        self.reg[i]
    }

    pub fn get_mut(&mut self, i: Register) -> &mut i16 {
        let i: usize = i.into();
        &mut self.reg[i]
    }
}
